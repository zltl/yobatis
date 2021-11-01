use super::info;
use mysql::prelude::*;
use mysql::*;
use regex::Regex;

// get the DDL for a database.
fn get_create_database_sql(con: &mut Conn, db_name: &str) -> Result<String> {
    let row: Row = con
        .query_first(format!("SHOW CREATE DATABASE `{}`", db_name))?
        .unwrap();
    let sql = row.get::<String, usize>(1).unwrap();

    let re = Regex::new(r"CREATE DATABASE").unwrap();
    let res_sql = re.replace(&sql, "CREATE DATABASE IF NOT EXISTS");

    return Result::Ok(String::from(res_sql));
}

// get all table name of a database.
fn get_table_list(con: &mut Conn, db_name: &str) -> Result<Vec<String>> {
    let mut tables: Vec<String> = Vec::new();
    let mut rows = con.query_iter(format!("SHOW TABLES FROM `{}`", db_name))?;

    while let Some(row) = rows.next() {
        let table = row?.get::<String, usize>(0).unwrap();
        tables.push(table);
    }

    return Result::Ok(tables);
}

// get table DDL for a table.
fn get_table_create_sql(con: &mut Conn, table_name: &str) -> Result<String> {
    let row: Row = con
        .query_first(format!("SHOW CREATE TABLE `{}`", table_name))?
        .unwrap();
    let sql = row.get::<String, usize>(1).unwrap();
    let re = Regex::new(r"CREATE TABLE").unwrap();
    let res_sql = re.replace(&sql, "CREATE TABLE IF NOT EXISTS");

    return Result::Ok(String::from(res_sql));
}

// parse a column type to info::ColumnType enum.
fn parse_column_type(column_type: &str) -> Result<info::ColumnType> {
    let reint = Regex::new(r".*(int|bit).*").unwrap();
    if reint.is_match(column_type) {
        return Ok(info::ColumnType::INT);
    }

    let refloat = Regex::new(r".*(float|dec|numeric|double).*").unwrap();
    if refloat.is_match(column_type) {
        return Ok(info::ColumnType::FLOAT);
    }

    return Ok(info::ColumnType::STRING);
}

// get column info from a table.
fn get_table_columns(con: &mut Conn, table_name: &str) -> Result<Vec<info::ColumnInfo>> {
    let mut columns: Vec<info::ColumnInfo> = Vec::new();
    let mut rows = con.query_iter(format!("SHOW FULL COLUMNS FROM `{}`", table_name))?;
    while let Some(row) = rows.next() {
        let column = row?;
        let column_name = column.get::<String, usize>(0).unwrap();
        let column_type = column.get::<String, usize>(1).unwrap();
        //let column_collation = column.get::<String, usize>(2).unwrap();
        let column_null = column.get::<String, usize>(3).unwrap();
        let column_key = column.get::<String, usize>(4).unwrap();
        let mut is_key = false;
        if column_key == "PRI" {
            is_key = true;
        }

        let column_default = column.get(5).unwrap();
        let mut column_default_str: String = String::new();
        match column_default {
            Value::NULL => {}
            _ => {
                column_default_str = from_value::<String>(column_default);
            }
        }

        //let column_extra = column.get::<String, usize>(6).unwrap();
        //let column_privileges = column.get::<String, usize>(7).unwrap();
        let column_comment = column.get::<String, usize>(8).unwrap();

        let column_info = info::ColumnInfo {
            name: column_name,
            type_: parse_column_type(&column_type)?,
            nullable: column_null == "YES",
            default: column_default_str,
            comment: column_comment,
            primary_key: is_key,
        };
        columns.push(column_info);
    }

    return Ok(columns);
}

// get table structure info.
fn get_table_info(con: &mut Conn, table_name: &str) -> Result<info::TableInfo> {
    return Result::Ok(info::TableInfo {
        name: table_name.to_string(),
        columns: get_table_columns(con, table_name)?,
        create: get_table_create_sql(con, table_name)?,
    });
}

// get all table structure info from a database.
fn get_all_table_info(con: &mut Conn, db_name: &str) -> Result<Vec<info::TableInfo>> {
    let mut tables: Vec<info::TableInfo> = Vec::new();

    let table_names = get_table_list(con, db_name)?;
    for table_name in table_names {
        let table_info = get_table_info(con, &table_name)?;
        tables.push(table_info);
    }

    return Result::Ok(tables);
}

// get all table and database structure info from a database.
pub fn get_info(opt: &info::DBOpt) -> Result<info::DBInfo> {
    let opts = OptsBuilder::new()
        .user(Some(opt.user.clone()))
        .pass(Some(opt.password.clone()))
        .ip_or_hostname(Some(opt.host.clone()))
        .tcp_port(opt.port as u16)
        .db_name(Some(opt.database.clone()));

    let mut con = Conn::new(opts)?;
    con.ping();

    return Ok(info::DBInfo {
        name: opt.database.clone(),
        create: get_create_database_sql(&mut con, &opt.database)?,
        tables: get_all_table_info(&mut con, &opt.database)?,
    });
}
