/// # this file parse all mapper xml files to Mapper struct
extern crate quick_xml;
extern crate serde;

use std::fs;
use std::path::Path;

use std::collections::HashMap;
use std::fmt;

use log::{debug, error, info, trace, warn};

/// result
#[derive(Debug, Clone)]
pub struct YoResult {
    pub column: String,
    pub property: String,
    pub yo_type: String,
}

/// <resultMap> - the mapper from sql query result column to C struct
#[derive(Debug, Clone)]
pub struct YoResultMap {
    pub id: String,
    pub type_: String,
    pub results: Vec<YoResult>,
}

/// <include> - embed a <sql> element to a mysql statement
#[derive(Debug)]
pub struct YoInclude {
    pub refid: String,
}

/// <sql> - the sql statement, or part of it.
#[derive(Debug)]
pub struct YoSql {
    pub id: String,
    pub text: String,
}

/// <if> - if condition for sql statement
#[derive(Debug)]
pub struct YoIf {
    pub test: String,
    pub content: Vec<Box<SqlElement>>,
}

/// <trim> - trim the sql statement
#[derive(Debug)]
pub struct YoTrim {
    pub prefix: String,
    pub suffix: String,
    pub suffix_overrides: String,
    pub prefix_overrides: String,
    pub content: Vec<Box<SqlElement>>,
}

/// <insert> - INSERT statement
#[derive(Debug)]
pub struct YoInsert {
    pub id: String,
    pub parameter_type: String,
    pub content: Vec<Box<SqlElement>>,
}

/// <update> - UPDATE statement
#[derive(Debug)]
pub struct YoUpdate {
    pub id: String,
    pub parameter_type: String,
    pub content: Vec<Box<SqlElement>>,
}

/// <select> - SELECT statement
#[derive(Debug)]
pub struct YoSelect {
    pub id: String,
    pub parameter_type: String,
    pub result_map: String,
    pub content: Vec<Box<SqlElement>>,
}

/// <delete> - DELETE statement
#[derive(Debug)]
pub struct YoDelete {
    pub id: String,
    pub parameter_type: String,
    pub content: Vec<Box<SqlElement>>,
}

/// option for sql statement elements
#[derive(Debug)]
pub enum SqlElement {
    YoInclude(YoInclude),
    YoText(String),
    YoTrim(YoTrim),
    YoIf(YoIf),
}

/// <mapper> - the root of an xml file
pub struct Mapper {
    pub namespace: String,
    pub result_maps: HashMap<String, YoResultMap>,
    pub type_maps: HashMap<String, YoResultMap>,
    pub sqls: HashMap<String, YoSql>,
    pub inserts: HashMap<String, YoInsert>,
    pub updates: HashMap<String, YoUpdate>,
    pub deletes: HashMap<String, YoDelete>,
    pub selects: HashMap<String, YoSelect>,
}

impl Mapper {
    pub fn new() -> Mapper {
        return Mapper {
            namespace: String::new(),
            result_maps: HashMap::new(),
            type_maps: HashMap::new(),
            sqls: HashMap::new(),
            inserts: HashMap::new(),
            updates: HashMap::new(),
            deletes: HashMap::new(),
            selects: HashMap::new(),
        };
    }
}

#[derive(Debug, Clone)]
pub struct ParseMapperError {
    message: String,
}

impl fmt::Display for ParseMapperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub type Result<T> = std::result::Result<T, ParseMapperError>;

/// parse <include>, <if>, <trim>
fn parse_sql_elements(node: &minidom::Element) -> Result<Vec<Box<SqlElement>>> {
    let mut elements = Vec::new();

    for child in node.nodes() {
        match child {
            minidom::Node::Element(element) => match element.name() {
                "include" => {
                    let refid = element.attr("refid").unwrap();
                    elements.push(Box::new(SqlElement::YoInclude(YoInclude {
                        refid: refid.to_string(),
                    })));
                }
                "if" => {
                    let test = element.attr("test").unwrap();
                    let contents = parse_sql_elements(element)?;
                    elements.push(Box::new(SqlElement::YoIf(YoIf {
                        test: test.to_string(),
                        content: contents,
                    })));
                }
                "trim" => {
                    let prefix = match element.attr("prefix") {
                        Some(suffix) => suffix.to_string(),
                        None => "".to_string(),
                    };
                    let suffix = match element.attr("suffix") {
                        Some(suffix) => suffix.to_string(),
                        None => "".to_string(),
                    };
                    let suffix_overrides = match element.attr("suffixOverrides") {
                        Some(suffix_overrides) => suffix_overrides.to_string(),
                        None => String::new(),
                    };
                    let prefix_overrides = match element.attr("prefixOverrides") {
                        Some(prefix_overrides) => prefix_overrides.to_string(),
                        None => String::new(),
                    };
                    let contents = parse_sql_elements(element)?;
                    elements.push(Box::new(SqlElement::YoTrim(YoTrim {
                        prefix: prefix.to_string(),
                        suffix: suffix.to_string(),
                        suffix_overrides: suffix_overrides.to_string(),
                        prefix_overrides: prefix_overrides.to_string(),
                        content: contents,
                    })));
                }
                _ => {
                    error!("unkown sql element: {:?}", element);
                }
            },
            minidom::Node::Text(text) => {
                elements.push(Box::new(SqlElement::YoText(text.to_string())));
            }

            _ => {}
        }
    }

    return Ok(elements);
}

// parse <sql>
fn parse_sql(node: &minidom::Element) -> Result<YoSql> {
    let id = node.attr("id").unwrap();
    let contents = parse_sql_elements(node)?;
    let text = contents
        .iter()
        .map(|e| match &**e {
            SqlElement::YoText(text) => text.to_string(),
            _ => String::new(),
        })
        .collect::<Vec<String>>()
        .join("");
    Ok(YoSql {
        id: id.to_string(),
        text: text,
    })
}

/// parse <resultMap>
fn parse_result_map(node: &minidom::Element) -> Result<YoResultMap> {
    let id = node.attr("id").unwrap();
    let type_ = node.attr("type").unwrap();
    let mut results = Vec::new();
    for child in node.children() {
        if child.name() == "result" {
            let column = child.attr("column").unwrap();
            let property = child.attr("property").unwrap();
            let yo_type = child.attr("yo_type").unwrap();
            results.push(YoResult {
                column: column.to_string(),
                property: property.to_string(),
                yo_type: yo_type.to_string(),
            });
        }
    }
    Ok(YoResultMap {
        id: id.to_string(),
        type_: type_.to_string(),
        results: results,
    })
}

/// parse <insert>
fn parse_insert(node: &minidom::Element) -> Result<YoInsert> {
    let id = node.attr("id").unwrap();
    let parameter_type = node.attr("parameterType").unwrap();
    let contents = parse_sql_elements(node)?;
    Ok(YoInsert {
        id: id.to_string(),
        parameter_type: parameter_type.to_string(),
        content: contents,
    })
}

/// parse <update>
fn parse_update(node: &minidom::Element) -> Result<YoUpdate> {
    let id = node.attr("id").unwrap();
    let parameter_type = node.attr("parameterType").unwrap();
    let contents = parse_sql_elements(node)?;
    Ok(YoUpdate {
        id: id.to_string(),
        parameter_type: parameter_type.to_string(),
        content: contents,
    })
}

/// parse <delete>
fn parse_delete(node: &minidom::Element) -> Result<YoDelete> {
    let id = node.attr("id").unwrap();
    let parameter_type = node.attr("parameterType").unwrap();
    let contents = parse_sql_elements(node)?;
    Ok(YoDelete {
        id: id.to_string(),
        parameter_type: parameter_type.to_string(),
        content: contents,
    })
}

/// parse <select>
fn parse_select(node: &minidom::Element) -> Result<YoSelect> {
    let id = node.attr("id").unwrap();
    let parameter_type = node.attr("parameterType").unwrap();
    let result_map = node.attr("resultMap").unwrap();
    let contents = parse_sql_elements(node)?;
    Ok(YoSelect {
        id: id.to_string(),
        parameter_type: parameter_type.to_string(),
        result_map: result_map.to_string(),
        content: contents,
    })
}

/// parse one mapper file
fn parse_mapper_file(path: &Path) -> Result<Mapper> {
    let mut mapper = Mapper::new();
    let contents = std::fs::read_to_string(path).unwrap();
    debug!("file: {:?}", path.to_str());
    let root: minidom::Element = contents.parse().unwrap();

    mapper.namespace = match root.attr("namespace") {
        Some(namespace) => namespace.to_string(),
        None => String::new(),
    };
    root.children().for_each(|child| match child.name() {
        "resultMap" => {
            let result_map = parse_result_map(&child).unwrap();
            debug!("result_map: {:?}", result_map);
            mapper
                .result_maps
                .insert(result_map.id.clone(), result_map.clone());
            mapper
                .type_maps
                .insert(result_map.type_.clone(), result_map.clone());
        }
        "sql" => {
            let sql = parse_sql(&child).unwrap();
            debug!("sql_elements: {:?}", sql);
            mapper.sqls.insert(sql.id.clone(), sql);
        }
        "insert" => {
            let insert = parse_insert(&child).unwrap();
            debug!("insert: {:?}", insert);
            mapper.inserts.insert(insert.id.clone(), insert);
        }
        "update" => {
            let update = parse_update(&child).unwrap();
            debug!("update: {:?}", update);
            mapper.updates.insert(update.id.clone(), update);
        }
        "delete" => {
            let delete = parse_delete(&child).unwrap();
            debug!("delete: {:?}", delete);
            mapper.deletes.insert(delete.id.clone(), delete);
        }
        "select" => {
            let select = parse_select(&child).unwrap();
            debug!("select: {:?}", select);
            mapper.selects.insert(select.id.clone(), select);
        }
        _ => {}
    });

    return Ok(mapper);
}

/// Parse all mapper files under folder dir and return a Mapper struct.
pub fn parse_mappers(dir: &str) -> Result<Vec<Mapper>> {
    let mut mapper_list = Vec::new();

    let entries = fs::read_dir(dir).unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if file_name.ends_with("-mapper.xml") {
            let mapper = parse_mapper_file(&path)?;
            mapper_list.push(mapper);
        }
    }

    return Ok(mapper_list);
}
