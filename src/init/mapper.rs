extern crate xml;
use super::info;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::path::Path;

use xml::writer::{EmitterConfig, EventWriter, Result, XmlEvent};

// generate db.xml.
// define the database.
fn gen_db_xml(inf: &info::DBInfo, path: &Path) -> Result<()> {
    let file = File::create(path.join("db.xml"))?;
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .autopad_comments(true)
        .create_writer(file);

    let event: XmlEvent = XmlEvent::start_element("db").into();
    writer.write(event)?;
    let event: XmlEvent = XmlEvent::start_element("name").into();
    writer.write(event)?;
    writer.write(XmlEvent::characters(&inf.name))?;
    let event: XmlEvent = XmlEvent::end_element().into(); // name
    writer.write(event)?;
    let event: XmlEvent = XmlEvent::start_element("create").into();
    writer.write(event)?;
    let event: XmlEvent = XmlEvent::characters(&inf.create).into();
    writer.write(event)?;
    let event: XmlEvent = XmlEvent::end_element().into(); // create
    writer.write(event)?;

    writer.write(XmlEvent::end_element())?; // db
    writer.write(XmlEvent::characters("\n"))?;
    return Ok(());
}

fn gen_col_type_str(t: &info::ColumnType) -> String {
    match t {
        info::ColumnType::INT => "int64_t".to_string(),
        info::ColumnType::STRING => "yb_string_t".to_string(),
        info::ColumnType::FLOAT => "double".to_string(),
    }
}

fn gen_col_type_null_str(t: &info::ColumnType) -> String {
    match t {
        info::ColumnType::INT => "YB_INT_NULL".to_string(),
        info::ColumnType::STRING => "YB_STRING_NULL".to_string(),
        info::ColumnType::FLOAT => "YB_FLOAT_NULL".to_string(),
    }
}

fn gen_result_map(
    inf: &info::TableInfo,
    name_norm: &str,
    writer: &mut EventWriter<File>,
) -> Result<()> {
    let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
    // begin resultMap
    let base_result_map_name = format!("yb_{}_t", name_norm);
    let result_map: XmlEvent = XmlEvent::start_element("resultMap")
        .attr("id", "BaseResultMap")
        .attr("type", &base_result_map_name)
        .into();
    writer.write(result_map)?;
    // results
    for col in &inf.columns {
        let col_name_norm = re.replace_all(&col.name, "_");
        let yo_type = gen_col_type_str(&col.type_);
        let result: XmlEvent = XmlEvent::start_element("result")
            .attr("column", &col.name)
            .attr("property", &col_name_norm)
            .attr("yo_type", &yo_type)
            .into();
        writer.write(result)?;
        let result: XmlEvent = XmlEvent::end_element().into();
        writer.write(result)?;
    }
    let result_map: XmlEvent = XmlEvent::end_element().into(); // resultMap
    writer.write(result_map)?;
    // end resultMap
    return Ok(());
}

fn gen_base_column_list(inf: &info::TableInfo, writer: &mut EventWriter<File>) -> Result<()> {
    let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
    let base_column_list: XmlEvent = XmlEvent::start_element("sql")
        .attr("id", "base_column_list")
        .into();
    writer.write(base_column_list)?;
    let mut first = true;
    for col in &inf.columns {
        if !first {
            writer.write(XmlEvent::characters(", "))?;
        }
        first = false;
        let col_name_norm = re.replace_all(&col.name, "_");
        let sql: XmlEvent = XmlEvent::characters(&col_name_norm).into();
        writer.write(sql)?;
    }
    let base_column_list: XmlEvent = XmlEvent::end_element().into();
    writer.write(base_column_list)?;

    return Ok(());
}

fn gen_insert_all(
    inf: &info::TableInfo,
    name_norm: &str,
    writer: &mut EventWriter<File>,
) -> Result<()> {
    let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
    let insert_name = format!("{}_insert", name_norm);
    let base_result_map_name = format!("yb_{}_t", name_norm);
    let insert: XmlEvent = XmlEvent::start_element("insert")
        .attr("id", &insert_name)
        .attr("parameterType", &base_result_map_name)
        .into();
    writer.write(insert)?;
    writer.write(XmlEvent::characters("INSERT INTO `"))?;
    writer.write(XmlEvent::characters(&inf.name))?;
    writer.write(XmlEvent::characters("` ("))?;
    let mut first = true;
    for col in &inf.columns {
        if !first {
            writer.write(XmlEvent::characters(", "))?;
        }
        first = false;
        let col_name_norm = re.replace_all(&col.name, "_");
        let sql: XmlEvent = XmlEvent::characters(&col_name_norm).into();
        writer.write(sql)?;
    }
    writer.write(XmlEvent::characters(") VALUES ("))?;
    first = true;
    for col in &inf.columns {
        if !first {
            writer.write(XmlEvent::characters(", "))?;
        }
        first = false;
        let col_name_norm = re.replace_all(&col.name, "_");
        let value_name_wrap = format!("#{{{}}}", col_name_norm);
        let sql: XmlEvent = XmlEvent::characters(&value_name_wrap).into();
        writer.write(sql)?;
    }
    writer.write(XmlEvent::characters(")"))?;
    let insert: XmlEvent = XmlEvent::end_element().into();
    writer.write(insert)?;

    return Ok(());
}

fn gen_delete_by_primary_key(
    inf: &info::TableInfo,
    name_norm: &str,
    key_name: &str,
    writer: &mut EventWriter<File>,
) -> Result<()> {
    let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
    let pri_name_norm = re.replace_all(&key_name, "_");
    let delete_name = format!("{}_delete_by_{}", name_norm, pri_name_norm);
    let mut key_type = &info::ColumnType::INT;
    for col in &inf.columns {
        if col.name == key_name {
            key_type = &col.type_;
            break;
        }
    }
    let key_type_str = gen_col_type_str(key_type);

    let delete: XmlEvent = XmlEvent::start_element("delete")
        .attr("id", &delete_name)
        .attr("parameterType", &key_type_str)
        .into();
    writer.write(delete)?;
    writer.write(XmlEvent::characters("DELETE FROM `"))?;
    writer.write(XmlEvent::characters(&inf.name))?;
    writer.write(XmlEvent::characters("` WHERE `"))?;
    writer.write(XmlEvent::characters(&key_name))?;
    writer.write(XmlEvent::characters("` = #{"))?;
    writer.write(XmlEvent::characters(&pri_name_norm))?;
    writer.write(XmlEvent::characters("}"))?;

    let delete: XmlEvent = XmlEvent::end_element().into();
    writer.write(delete)?;

    return Ok(());
}

fn gen_select_by_primary_key(
    inf: &info::TableInfo,
    name_norm: &str,
    key_name: &str,
    writer: &mut EventWriter<File>,
) -> Result<()> {
    let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
    let pri_name_norm = re.replace_all(&key_name, "_");
    let select_name = format!("{}_select_by_{}", name_norm, pri_name_norm);

    let mut key_type = &info::ColumnType::INT;
    for col in &inf.columns {
        if col.name == key_name {
            key_type = &col.type_;
            break;
        }
    }
    let key_type_str = gen_col_type_str(key_type);

    let select: XmlEvent = XmlEvent::start_element("select")
        .attr("id", &select_name)
        .attr("resultMap", "BaseResultMap")
        .attr("parameterType", &key_type_str)
        .into();
    writer.write(select)?;
    writer.write(XmlEvent::characters("SELECT "))?;
    let include: XmlEvent = XmlEvent::start_element("include")
        .attr("refid", "base_column_list")
        .into();
    writer.write(include)?;
    let select: XmlEvent = XmlEvent::end_element().into();
    writer.write(select)?;

    writer.write(XmlEvent::characters(" FROM `"))?;
    writer.write(XmlEvent::characters(&inf.name))?;
    writer.write(XmlEvent::characters("` WHERE "))?;
    let where_cond = format!("{} = #{{{}}}", key_name, pri_name_norm);
    let sql: XmlEvent = XmlEvent::characters(&where_cond).into();
    writer.write(sql)?;

    let select: XmlEvent = XmlEvent::end_element().into();
    writer.write(select)?;

    return Ok(());
}

fn gen_update_by_primary_key(
    inf: &info::TableInfo,
    name_norm: &str,
    key_name: &str,
    writer: &mut EventWriter<File>,
) -> Result<()> {
    let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
    let pri_name_norm = re.replace_all(&key_name, "_");
    let update_name = format!("{}_update_by_{}", name_norm, pri_name_norm);
    let base_result_map_name = format!("yb_{}_t", name_norm);

    let update: XmlEvent = XmlEvent::start_element("update")
        .attr("id", &update_name)
        .attr("parameterType", &base_result_map_name)
        .into();
    writer.write(update)?;
    writer.write(XmlEvent::characters("UPDATE `"))?;
    writer.write(XmlEvent::characters(&inf.name))?;
    writer.write(XmlEvent::characters("` SET "))?;
    let mut first = true;
    for col in &inf.columns {
        if !first {
            writer.write(XmlEvent::characters(", "))?;
        }
        first = false;
        let col_name_norm = re.replace_all(&col.name, "_");
        let value_name_wrap = format!("`{}` = #{{{}}}", col.name, col_name_norm);
        let sql: XmlEvent = XmlEvent::characters(&value_name_wrap).into();
        writer.write(sql)?;
    }
    writer.write(XmlEvent::characters(" WHERE "))?;
    let sql: XmlEvent = XmlEvent::characters(&pri_name_norm).into();
    writer.write(sql)?;
    writer.write(XmlEvent::characters(" = #{"))?;
    let sql: XmlEvent = XmlEvent::characters(&pri_name_norm).into();
    writer.write(sql)?;
    writer.write(XmlEvent::characters("}"))?;
    let update: XmlEvent = XmlEvent::end_element().into();
    writer.write(update)?;

    return Ok(());
}

fn gen_update_by_primary_key_selective(
    inf: &info::TableInfo,
    name_norm: &str,
    key_name: &str,
    writer: &mut EventWriter<File>,
) -> Result<()> {
    let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
    let pri_name_norm = re.replace_all(&key_name, "_");
    let update_name = format!("{}_update_by_{}_selective", name_norm, pri_name_norm);
    let base_result_map_name = format!("yb_{}_t", name_norm);

    let update: XmlEvent = XmlEvent::start_element("update")
        .attr("id", &update_name)
        .attr("parameterType", &base_result_map_name)
        .into();
    writer.write(update)?;
    writer.write(XmlEvent::characters("UPDATE `"))?;
    writer.write(XmlEvent::characters(&inf.name))?;
    writer.write(XmlEvent::characters("` SET "))?;

    let trim: XmlEvent = XmlEvent::start_element("trim")
        .attr("suffixOverrides", ",")
        .into();
    writer.write(trim)?;
    for col in &inf.columns {
        if col.name == key_name {
            continue;
        }

        let col_name_norm = re.replace_all(&col.name, "_");
        let value_name_wrap = format!("`{}` = #{{{}}}, ", col.name, col_name_norm);
        let test_cond = format!("{} != {}", col_name_norm, gen_col_type_null_str(&col.type_));

        let if_: XmlEvent = XmlEvent::start_element("if")
            .attr("test", &test_cond)
            .into();
        writer.write(if_)?;
        let sql: XmlEvent = XmlEvent::characters(&value_name_wrap).into();
        writer.write(sql)?;
        let if_: XmlEvent = XmlEvent::end_element().into();
        writer.write(if_)?;
    }
    let trim: XmlEvent = XmlEvent::end_element().into();
    writer.write(trim)?;

    writer.write(XmlEvent::characters(" WHERE "))?;
    let sql: XmlEvent = XmlEvent::characters(&pri_name_norm).into();
    writer.write(sql)?;
    writer.write(XmlEvent::characters(" = #{"))?;
    let sql: XmlEvent = XmlEvent::characters(&pri_name_norm).into();
    writer.write(sql)?;
    writer.write(XmlEvent::characters("}"))?;
    let update: XmlEvent = XmlEvent::end_element().into();
    writer.write(update)?;

    return Ok(());
}

fn gen_insert_selective(
    inf: &info::TableInfo,
    name_norm: &str,
    writer: &mut EventWriter<File>,
) -> Result<()> {
    let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
    let insert_name = format!("{}_insert_selective", name_norm);
    let base_result_map_name = format!("yb_{}_t", name_norm);

    let insert: XmlEvent = XmlEvent::start_element("insert")
        .attr("id", &insert_name)
        .attr("parameterType", &base_result_map_name)
        .into();
    writer.write(insert)?;
    writer.write(XmlEvent::characters("INSERT INTO `"))?;
    writer.write(XmlEvent::characters(&inf.name))?;
    writer.write(XmlEvent::characters("` "))?;
    // columns
    let insert_selective_columns: XmlEvent = XmlEvent::start_element("trim")
        .attr("prefix", "(")
        .attr("suffix", ")")
        .attr("suffixOverrides", ",")
        .into();
    writer.write(insert_selective_columns)?;
    for col in &inf.columns {
        let col_name_norm = re.replace_all(&col.name, "_");
        let col_condition = &format!("{} != {}", col_name_norm, gen_col_type_null_str(&col.type_));
        let col_selective: XmlEvent = XmlEvent::start_element("if")
            .attr("test", col_condition)
            .into();
        writer.write(col_selective)?;
        let sql: XmlEvent = XmlEvent::characters(&col_name_norm).into();
        writer.write(sql)?;
        writer.write(XmlEvent::characters(","))?;
        let col_selective: XmlEvent = XmlEvent::end_element().into();
        writer.write(col_selective)?;
    }
    let insert_selective_columns: XmlEvent = XmlEvent::end_element().into();
    writer.write(insert_selective_columns)?;

    // values
    let insert_selective_values: XmlEvent = XmlEvent::start_element("trim")
        .attr("prefix", "VALUES (")
        .attr("suffix", ")")
        .attr("suffixOverrides", ",")
        .into();
    writer.write(insert_selective_values)?;
    for col in &inf.columns {
        let col_name_norm = re.replace_all(&col.name, "_");
        let col_condition = &format!("{} != {}", col_name_norm, gen_col_type_null_str(&col.type_));
        let col_selective: XmlEvent = XmlEvent::start_element("if")
            .attr("test", col_condition)
            .into();
        writer.write(col_selective)?;
        let value_name_wrap = format!("#{{{}}}", col_name_norm);
        let sql: XmlEvent = XmlEvent::characters(&value_name_wrap).into();
        writer.write(sql)?;
        writer.write(XmlEvent::characters(","))?;
        let col_selective: XmlEvent = XmlEvent::end_element().into();
        writer.write(col_selective)?;
    }
    let insert_selective_values: XmlEvent = XmlEvent::end_element().into();
    writer.write(insert_selective_values)?;

    let insert: XmlEvent = XmlEvent::end_element().into();
    writer.write(insert)?;

    return Ok(());
}

// generate table-mapper.xml
// define a table and query mapper.
fn gen_table_xml(inf: &info::TableInfo, path: &Path) -> Result<()> {
    let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
    let name_norm = re.replace_all(&inf.name, "_");
    let filename = format!("{}-mapper.xml", name_norm);
    let namespace = format!("{}_mapper", name_norm);

    let file = File::create(path.join(filename))?;
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .autopad_comments(true)
        .create_writer(file);

    // begin mapper
    let mapper: XmlEvent = XmlEvent::start_element("mapper")
        .attr("namespace", &namespace)
        .into();
    writer.write(mapper)?;

    gen_result_map(inf, &name_norm, &mut writer)?;

    gen_base_column_list(inf, &mut writer)?;

    gen_insert_all(&inf, &name_norm, &mut writer)?;

    gen_insert_selective(&inf, &name_norm, &mut writer)?;

    for col in &inf.columns {
        if col.primary_key {
            gen_update_by_primary_key(inf, &name_norm, &col.name, &mut writer)?;
            gen_update_by_primary_key_selective(inf, &name_norm, &col.name, &mut writer)?;

            gen_select_by_primary_key(inf, &name_norm, &col.name, &mut writer)?;

            gen_delete_by_primary_key(inf, &name_norm, &col.name, &mut writer)?;
        }
    }

    let mapper: XmlEvent = XmlEvent::end_element().into(); // mapper
    writer.write(mapper)?;
    // end mapper

    writer.write(XmlEvent::characters("\n"))?;

    return Ok(());
}

// generate the mapper xml files.
pub fn generate(inf: &info::DBInfo, dir: &str) -> Result<()> {
    fs::create_dir_all(dir)?;
    let path = Path::new(dir);

    gen_db_xml(inf, &path)?;

    for table in &inf.tables {
        gen_table_xml(table, &path)?;
    }

    return Ok(());
}
