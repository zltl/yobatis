/// # This file generates C codes from Mapper structs
use std::fs;
use std::path::Path;

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use super::mapper;
use log::{debug, error, info, trace, warn};

/// error type
#[derive(Debug, Clone)]
pub struct GenCError {
    message: String,
}

impl fmt::Display for GenCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
pub type Result<T> = std::result::Result<T, GenCError>;

/// write command files
fn gen_common(path: &Path) -> Result<()> {
    let yb_common_c = include_bytes!("../../yb_common/src/yb_common.c");
    let yb_common_h = include_bytes!("../../yb_common/src/yb_common.h");

    let mut common_c_file = File::create(path.join("yb_common.c")).unwrap();
    let mut common_h_file = File::create(path.join("yb_common.h")).unwrap();
    common_c_file.write_all(yb_common_c).unwrap();
    common_h_file.write_all(yb_common_h).unwrap();

    return Ok(());
}

/// write header guard
fn write_guard_start(mapper_h_file: &mut File, mapper: &mapper::Mapper) -> Result<()> {
    // #ifndef YB_XXXXX_MAPPER_H__
    // #define YB_XXXXX_MAPPER_H__
    let pp_guard = format!("YB_{}_H__", mapper.namespace.to_uppercase());
    mapper_h_file.write("#ifndef ".as_bytes()).unwrap();
    mapper_h_file.write(pp_guard.as_bytes()).unwrap();
    mapper_h_file.write("\n".as_bytes()).unwrap();
    mapper_h_file.write("#define ".as_bytes()).unwrap();
    mapper_h_file.write(pp_guard.as_bytes()).unwrap();
    mapper_h_file.write("\n\n".as_bytes()).unwrap();

    return Ok(());
}

fn write_guard_end(mapper_h_file: &mut File, mapper: &mapper::Mapper) -> Result<()> {
    // #endif // YB_XXXXX_MAPPER_H__
    let pp_guard = format!("YB_{}_H__", mapper.namespace.to_uppercase());
    mapper_h_file.write("#endif // ".as_bytes()).unwrap();
    mapper_h_file.write(pp_guard.as_bytes()).unwrap();
    mapper_h_file.write("\n\n".as_bytes()).unwrap();

    return Ok(());
}

fn write_includes(mapper_c_file: &mut File, filename_h: &str) -> Result<()> {
    mapper_c_file.write("#include \"".as_bytes()).unwrap();
    mapper_c_file.write(filename_h.as_bytes()).unwrap();
    mapper_c_file.write("\"\n\n".as_bytes()).unwrap();

    mapper_c_file
        .write("#include \"yb_common.h\"\n\n".as_bytes())
        .unwrap();

    return Ok(());
}

fn write_result_map_define(
    mapper_h_file: &mut File,
    mapper_c_file: &mut File,
    result_map: &mapper::YoResultMap,
) -> Result<()> {
    // struct
    let def_line = format!("struct {}_s {{\n", result_map.type_);
    mapper_h_file.write(def_line.as_bytes()).unwrap();
    for result in &result_map.results {
        let member_line = format!("    {} {};\n", result.yo_type, result.property);
        mapper_h_file.write(member_line.as_bytes()).unwrap();
    }
    mapper_h_file.write("};\n".as_bytes()).unwrap();
    // typedef
    let tydef = format!(
        "typedef struct {}_s* {};\n",
        result_map.type_, result_map.type_
    );
    mapper_h_file.write(tydef.as_bytes()).unwrap();

    return Ok(());
}

fn write_result_map_new(
    mapper_h_file: &mut File,
    mapper_c_file: &mut File,
    result_map: &mapper::YoResultMap,
) -> Result<()> {
    // declare new
    let new_fn = format!("{} {}_new();\n", result_map.type_, result_map.type_);
    mapper_h_file.write(new_fn.as_bytes()).unwrap();

    // impl init
    let new_fn_line = format!("{} {}_new() {{\n", result_map.type_, result_map.type_);
    mapper_c_file.write(new_fn_line.as_bytes()).unwrap();
    let malloc_line = format!(
        "    {} n = ({})malloc(sizeof(struct {}_s));\n",
        result_map.type_, result_map.type_, result_map.type_
    );
    mapper_c_file.write(malloc_line.as_bytes()).unwrap();
    for result in &result_map.results {
        let line;
        match result.yo_type.as_str() {
            "int64_t" => {
                line = format!("    n->{} = YB_INT_NULL;\n", result.property);
            }
            "double" => {
                line = format!("    n->{} = YB_FLOAT_NULL;\n", result.property);
            }
            "yb_string_t" => {
                line = format!("    n->{} = YB_STRING_NULL;\n", result.property);
            }
            _ => {
                return Err(GenCError {
                    message: format!("unsupported type: {}", result.yo_type),
                });
            }
        }
        mapper_c_file.write(line.as_bytes()).unwrap();
    }
    mapper_c_file
        .write("    return n;\n}\n\n".as_bytes())
        .unwrap();
    return Ok(());
}

fn write_result_map_free(
    mapper_h_file: &mut File,
    mapper_c_file: &mut File,
    result_map: &mapper::YoResultMap,
) -> Result<()> {
    // declare free
    let free_fn = format!("void {}_free({});\n", result_map.type_, result_map.type_);
    mapper_h_file.write(free_fn.as_bytes()).unwrap();
    // impl free
    let free_fn_line = format!(
        "void {}_free({} n) {{\n",
        result_map.type_, result_map.type_
    );
    mapper_c_file.write(free_fn_line.as_bytes()).unwrap();

    for result in &result_map.results {
        let val_name = &result.property;
        match result.yo_type.as_str() {
            "yb_string_t" => {
                let free_string_stmt = format!(
                    "    if (n->{} != YB_STRING_NULL) {{\n        yb_string_free(n->{});\n    }}\n",
                    val_name, val_name
                );
                mapper_c_file.write(free_string_stmt.as_bytes()).unwrap();
            }
            _ => {}
        }
    }
    mapper_c_file
        .write("    free(n);\n}\n\n".as_bytes())
        .unwrap();

    return Ok(());
}

fn spaces(n: usize) -> String {
    return std::iter::repeat(' ').take(n).collect::<String>();
}

/// write <if> statement,
fn write_append_if_stmt(
    mapper_c_file: &mut File,
    indent: usize,
    table: &mapper::Mapper,
    elem: &mapper::YoIf,
    valname: &String,
    inc: &mut i32,
) -> Result<()> {
    // if (n->???? == ????) {
    //     generate <tmp> content ...
    //     yb_string_append_c_str(<valname>, "<tmp>...");
    //     yb_string_free(<tmp>);
    // }
    let line = format!("{}if (n->{}) {{\n", spaces(indent), elem.test);
    mapper_c_file.write(line.as_bytes()).unwrap();
    let tmp_val = format!("tmp_{}", *inc);
    *inc += 1;
    let line = format!(
        "{}yb_string_t {} = yb_string_new();\n",
        spaces(indent + 4),
        tmp_val
    );
    mapper_c_file.write(line.as_bytes()).unwrap();

    write_sql_gen_stmt(
        mapper_c_file,
        indent + 4,
        table,
        &elem.content,
        &tmp_val,
        inc,
    )?;

    // append
    let line = format!(
        "{}yb_string_append({}, {});\n",
        spaces(indent + 4),
        valname,
        tmp_val
    );
    mapper_c_file.write(line.as_bytes()).unwrap();

    // free tmp
    let line = format!("{}yb_string_free({});\n", spaces(indent + 4), tmp_val);
    mapper_c_file.write(line.as_bytes()).unwrap();

    // }
    let line = format!("{}}}\n", spaces(indent));
    mapper_c_file.write(line.as_bytes()).unwrap();

    return Ok(());
}

fn write_trim_stmt(
    mapper_c_file: &mut File,
    indent: usize,
    table: &mapper::Mapper,
    elem: &mapper::YoTrim,
    valname: &String,
    inc: &mut i32,
) -> Result<()> {
    // yb_string_t tmp_<inc> = yb_string_new();
    // generate <tmp_inc> content ...
    // yb_string_trim(...) --> valname;
    // yb_string_free(tmp_inc);
    let tmp_val = format!("tmp_{}", *inc);
    *inc += 1;
    let line = format!(
        "{}yb_string_t {} = yb_string_new();\n",
        spaces(indent),
        tmp_val
    );
    mapper_c_file.write(line.as_bytes()).unwrap();

    write_sql_gen_stmt(mapper_c_file, indent, table, &elem.content, &tmp_val, inc)?;

    let line = format!(
        "{space}yb_string_trim({src}, \"{prefix}\", \"{suffix}\", \"{prefix_override}\", \"{suffix_override}\", {dest});\n",
        space=spaces(indent),
        src=tmp_val,
        prefix=elem.prefix,
        suffix=elem.suffix,
        prefix_override=elem.prefix_overrides,
        suffix_override=elem.suffix_overrides,
        dest=valname
    );
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!("{}yb_string_free({});\n", spaces(indent), tmp_val);
    mapper_c_file.write(line.as_bytes()).unwrap();

    return Ok(());
}

fn write_sql_gen_stmt(
    mapper_c_file: &mut File,
    indent: usize,
    table: &mapper::Mapper,
    elems: &Vec<Box<mapper::SqlElement>>,
    valname: &String,
    inc: &mut i32,
) -> Result<()> {
    for elem in elems {
        match &**elem {
            mapper::SqlElement::YoIf(ref if_elem) => {
                write_append_if_stmt(mapper_c_file, indent, table, if_elem, valname, inc)?;
            }
            mapper::SqlElement::YoInclude(inc) => {
                let inc_sql = match table.sqls.get(&inc.refid) {
                    Some(sql) => sql,
                    None => {
                        return Err(GenCError {
                            message: format!("include sql not found: {}", inc.refid),
                        });
                    }
                };
                let line = format!(
                    "{}yb_string_append_c_str({}, \"{}\")",
                    spaces(indent),
                    valname,
                    inc_sql.text.replace("\n", "\\n").replace("\r", "\\r")
                );
                mapper_c_file.write(line.as_bytes()).unwrap();
            }
            mapper::SqlElement::YoText(ref tex) => {
                let line = format!(
                    "{}yb_string_append_c_str({}, \"{}\");\n",
                    spaces(indent),
                    valname,
                    tex.replace("\n", "\\n").replace("\r", "\\r")
                );
                mapper_c_file.write(line.as_bytes()).unwrap();
            }
            mapper::SqlElement::YoTrim(ref elem) => {
                write_trim_stmt(mapper_c_file, indent, table, elem, valname, inc)?;
            }
        }
    }

    return Ok(());
}

fn write_cmd_to_prepare_sql(
    mapper_c_file: &mut File,
    result_map: &mapper::YoResultMap,
    table: &mapper::Mapper,
) -> Result<()> {
    // for
    let line = format!(
        "{}for (int64_t i = 0; i < yb_string_length(cmd); ++i) {{\n",
        spaces(4)
    );
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!(
        "{}if (i < yb_string_length(cmd) - 1 && yb_string_data(cmd)[i] == '#' && yb_string_data(cmd)[i+1] == '{{') {{\n",
        spaces(8)
    );
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!(
        "{}yb_string_append_data(prepare_sql, yb_string_data(cmd)+pre, i-pre);\n{}pre = i + 2;\n",
        spaces(12),
        spaces(12)
    );
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!(
        "{}while (pre < yb_string_length(cmd) && yb_string_data(cmd)[pre] != '}}') {{\n{}++pre;\n{}}}\n",
        spaces(12),
        spaces(16),
        spaces(12)
    );
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!(
        "{}yb_string_t val_tmp = yb_string_new();\n{}yb_string_ref_data(val_tmp, yb_string_data(cmd)+i+2, pre - i - 2);\n",
        spaces(12),
        spaces(12)
    );
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!("{}i = pre;\n", spaces(12));
    mapper_c_file.write(line.as_bytes()).unwrap();

    for row in &result_map.results {
        let line = format!(
            "{}if (yb_string_compare_cstr(val_tmp, \"{}\")) {{\n",
            spaces(12),
            row.property
        );
        mapper_c_file.write(line.as_bytes()).unwrap();
        let line = format!(
            "{}yb_string_append_c_str(prepare_sql, \"?\");\n",
            spaces(16),
        );
        mapper_c_file.write(line.as_bytes()).unwrap();

        // bind
        if row.yo_type == "int64_t" {
            let line = format!(
                "
                bind[bind_num].buffer_type = MYSQL_TYPE_LONGLONG;
                bind[bind_num].buffer = &n->{};
                ++bind_num;\n",
                row.property,
            );
            mapper_c_file.write(line.as_bytes()).unwrap();
        } else if row.yo_type == "double" {
            let line = format!(
                "
                bind[bind_num].buffer_type = MYSQL_TYPE_DOUBLE;
                bind[bind_num].buffer = &n->{};
                ++bind_num;\n",
                row.property,
            );
            mapper_c_file.write(line.as_bytes()).unwrap();
        } else {
            let line = format!(
                "
                bind[bind_num].buffer_type = MYSQL_TYPE_STRING;
                bind[bind_num].buffer = yb_string_data(n->{field});
                bind[bind_num].buffer_length = yb_string_length(n->{field});
                ++bind_num;\n",
                field = row.property
            );
            mapper_c_file.write(line.as_bytes()).unwrap();
        }

        let line = format!("{}}}\n", spaces(12));
        mapper_c_file.write(line.as_bytes()).unwrap();
    }
    let line = format!("{}}}\n", spaces(8));
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!("{}}}\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!(
        "{}yb_string_append_data(prepare_sql, yb_string_data(cmd)+pre, yb_string_length(cmd)-pre);\n",
        spaces(4)
    );
    mapper_c_file.write(line.as_bytes()).unwrap();
    return Ok(());
}

fn write_insert_fn(
    mapper_h_file: &mut File,
    mapper_c_file: &mut File,
    insert_m: &mapper::YoInsert,
    table: &mapper::Mapper,
) -> Result<()> {
    {
        // header file
        // declare insert
        let insert_fn = format!(
            "int {}(MYSQL* conn, {} n);\n",
            insert_m.id, insert_m.parameter_type
        );
        mapper_h_file.write(insert_fn.as_bytes()).unwrap();
    }

    // impl insert
    //      int <insert_id>(MYSQL* conn, <parameter_type> n) {
    let insert_fn_line = format!(
        "int {}(MYSQL* conn, {} n) {{\n",
        insert_m.id, insert_m.parameter_type
    );
    mapper_c_file.write(insert_fn_line.as_bytes()).unwrap();
    // statements generating
    //          yb_string_t sql = yb_string_new();
    let line = format!("{}yb_string_t cmd = yb_string_new();\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    // generate cmd="INSERT INTO XXX(a,b,c) VALUES (#{xxx}, #{yyy}, #{xxx})"
    write_sql_gen_stmt(
        mapper_c_file,
        4,
        table,
        &insert_m.content,
        &String::from("cmd"),
        &mut 0,
    )?;

    let result_map = table.type_maps.get(&insert_m.parameter_type).unwrap();

    let line = format!(
        "{}MYSQL_BIND bind[{}*2];\n",
        spaces(4),
        result_map.results.len()
    );
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!(
        "{}memset(bind, 0, sizeof(MYSQL_BIND)*{}*2);\n",
        spaces(4),
        result_map.results.len()
    );
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}int bind_num = 0;\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!("{}yb_string_t prepare_sql = yb_string_new();\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}int64_t pre = 0;\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    // #{} -> ? sql prepare
    write_cmd_to_prepare_sql(mapper_c_file, result_map, table)?;

    let line = format!("{}MYSQL_STMT* stmt = mysql_stmt_init(NULL);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!(
        "{}mysql_stmt_prepare(stmt, yb_string_data(prepare_sql), yb_string_length(prepare_sql));\n",
        spaces(4)
    );
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}mysql_stmt_bind_param(stmt, bind);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}mysql_stmt_execute(stmt);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}mysql_stmt_close(stmt);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!("{}yb_string_free(prepare_sql);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}yb_string_free(cmd);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    mapper_c_file.write(line.as_bytes()).unwrap();

    mapper_c_file
        .write("    return YB_OK;\n}\n\n".as_bytes())
        .unwrap();
    return Ok(());
}

fn write_update_fn(
    mapper_h_file: &mut File,
    mapper_c_file: &mut File,
    update_m: &mapper::YoUpdate,
    table: &mapper::Mapper,
) -> Result<()> {
    {
        // header file
        // declare insert
        let line = format!(
            "int {}(MYSQL* conn, {} n);\n",
            update_m.id, update_m.parameter_type
        );
        mapper_h_file.write(line.as_bytes()).unwrap();
    }

    // impl insert
    //      int <insert_id>(MYSQL* conn, <parameter_type> n) {
    let insert_fn_line = format!(
        "int {}(MYSQL* conn, {} n) {{\n",
        update_m.id, update_m.parameter_type
    );
    mapper_c_file.write(insert_fn_line.as_bytes()).unwrap();
    // statements generating
    //          yb_string_t sql = yb_string_new();
    let line = format!("{}yb_string_t cmd = yb_string_new();\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    // generate cmd="UPDATE XXX FFF SET a=#{xxx}, b=#{yyy}"
    write_sql_gen_stmt(
        mapper_c_file,
        4,
        table,
        &update_m.content,
        &String::from("cmd"),
        &mut 0,
    )?;

    let result_map = table.type_maps.get(&update_m.parameter_type).unwrap();

    let line = format!(
        "{}MYSQL_BIND bind[{}*2];\n",
        spaces(4),
        result_map.results.len()
    );
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!(
        "{}memset(bind, 0, sizeof(MYSQL_BIND)*{}*2);\n",
        spaces(4),
        result_map.results.len()
    );
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}int bind_num = 0;\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!("{}yb_string_t prepare_sql = yb_string_new();\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}int64_t pre = 0;\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    // #{} -> ? sql prepare
    write_cmd_to_prepare_sql(mapper_c_file, result_map, table)?;

    let line = format!("{}MYSQL_STMT* stmt = mysql_stmt_init(NULL);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!(
        "{}mysql_stmt_prepare(stmt, yb_string_data(prepare_sql), yb_string_length(prepare_sql));\n",
        spaces(4)
    );
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}mysql_stmt_bind_param(stmt, bind);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}mysql_stmt_execute(stmt);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}mysql_stmt_close(stmt);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!("{}yb_string_free(prepare_sql);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}yb_string_free(cmd);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    mapper_c_file.write(line.as_bytes()).unwrap();

    mapper_c_file
        .write("    return YB_OK;\n}\n\n".as_bytes())
        .unwrap();
    return Ok(());
}

fn write_select_fn(
    mapper_h_file: &mut File,
    mapper_c_file: &mut File,
    select_m: &mapper::YoSelect,
    table: &mapper::Mapper,
) -> Result<()> {
    let result_out_map = table.result_maps.get(&select_m.result_map).unwrap();

    {
        // header file
        // declare insert
        let line = format!(
            "int {}(MYSQL* conn, {} n, {} out);\n",
            select_m.id, select_m.parameter_type, result_out_map.type_
        );
        mapper_h_file.write(line.as_bytes()).unwrap();
    }

    // impl insert
    //      int <insert_id>(MYSQL* conn, <parameter_type> n) {
    let insert_fn_line = format!(
        "int {}(MYSQL* conn, {} n) {{\n",
        select_m.id, select_m.parameter_type
    );
    mapper_c_file.write(insert_fn_line.as_bytes()).unwrap();
    // statements generating
    //          yb_string_t sql = yb_string_new();
    let line = format!("{}yb_string_t cmd = yb_string_new();\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    // generate cmd="UPDATE XXX FFF SET a=#{xxx}, b=#{yyy}"
    write_sql_gen_stmt(
        mapper_c_file,
        4,
        table,
        &select_m.content,
        &String::from("cmd"),
        &mut 0,
    )?;

    let result_map = table.type_maps.get(&select_m.parameter_type).unwrap();

    let line = format!(
        "{}MYSQL_BIND bind[{}*2];\n",
        spaces(4),
        result_map.results.len()
    );
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!(
        "{}memset(bind, 0, sizeof(MYSQL_BIND)*{}*2);\n",
        spaces(4),
        result_map.results.len()
    );
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}int bind_num = 0;\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!(
        "{}MYSQL_BIND bind_out[{}*2];\n",
        spaces(4),
        result_out_map.results.len()
    );
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!(
        "{}memset(bind, 0, sizeof(MYSQL_BIND)*{}*2);\n",
        spaces(4),
        result_out_map.results.len()
    );
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}int bind_out_num = 0;\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!("{}yb_string_t prepare_sql = yb_string_new();\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}int64_t pre = 0;\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    // #{} -> ? sql prepare
    // and bind in
    write_cmd_to_prepare_sql(mapper_c_file, result_map, table)?;

    let line = format!("{}MYSQL_STMT* stmt = mysql_stmt_init(NULL);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!(
        "{}mysql_stmt_prepare(stmt, yb_string_data(prepare_sql), yb_string_length(prepare_sql));\n",
        spaces(4)
    );
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}mysql_stmt_bind_param(stmt, bind);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!("{}mysql_stmt_execute(stmt);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    // TODO: check result

    let line = format!("{}mysql_stmt_close(stmt);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    let line = format!("{}yb_string_free(prepare_sql);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();
    let line = format!("{}yb_string_free(cmd);\n", spaces(4));
    mapper_c_file.write(line.as_bytes()).unwrap();

    mapper_c_file.write(line.as_bytes()).unwrap();

    mapper_c_file
        .write("    return YB_OK;\n}\n\n".as_bytes())
        .unwrap();
    return Ok(());
}

fn write_result_map(
    mapper_h_file: &mut File,
    mapper_c_file: &mut File,
    result_map: &mapper::YoResultMap,
) -> Result<()> {
    debug!(
        "writing result map: {}::{}",
        result_map.id, result_map.type_
    );

    write_result_map_define(mapper_h_file, mapper_c_file, result_map)?;
    write_result_map_new(mapper_h_file, mapper_c_file, result_map)?;
    write_result_map_free(mapper_h_file, mapper_c_file, result_map)?;
    return Ok(());
}

fn gen_mapper_src(path: &Path, mapper: &mapper::Mapper) -> Result<()> {
    let filename_c = format!("yb_{}.c", mapper.namespace);
    let filename_h = format!("yb_{}.h", mapper.namespace);
    let mut mapper_c_file = File::create(path.join(&filename_c)).unwrap();
    let mut mapper_h_file = File::create(path.join(&filename_h)).unwrap();

    write_guard_start(&mut mapper_h_file, mapper)?;
    write_includes(&mut mapper_c_file, &filename_h)?;

    for (_, result_map) in &mapper.result_maps {
        write_result_map(&mut mapper_h_file, &mut mapper_c_file, &result_map)?;
    }

    for (_, insert) in &mapper.inserts {
        write_insert_fn(&mut mapper_h_file, &mut mapper_c_file, &insert, &mapper)?;
    }

    for (_, update) in &mapper.updates {
        write_update_fn(&mut mapper_h_file, &mut mapper_c_file, &update, &mapper)?;
    }

    for (_, select) in &mapper.selects {
        write_select_fn(&mut mapper_h_file, &mut mapper_c_file, &select, &mapper)?;
    }

    write_guard_end(&mut mapper_h_file, mapper)?;
    return Ok(());
}

pub fn gen_c(mappers: Vec<mapper::Mapper>, dir: &str) -> Result<()> {
    let path = Path::new(dir);
    fs::create_dir_all(dir).unwrap();

    gen_common(path).unwrap();

    for table in mappers {
        gen_mapper_src(path, &table).unwrap();
    }

    return Ok(());
}
