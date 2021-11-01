/// # The Mapper Info struct to describe table of mysql

/// Date type of table column.
/// Only 3 types support:
/// All integer types tag as INT.
/// All string, text, blob, date ... types tag as STRING
/// All floating point number tag as FLOAT
pub enum ColumnType {
    INT,
    STRING,
    FLOAT,
}

/// Column description
pub struct ColumnInfo {
    /// Column name
    pub name: String,
    /// Column type, see [`ColumnType`](enum.ColumnType.html)
    pub type_: ColumnType,
    /// If column can be null
    pub nullable: bool,
    /// the default value
    pub default: String,
    /// the comment of column
    pub comment: String,
    /// If this column is primary key
    pub primary_key: bool,
}

/// Table description
pub struct TableInfo {
    /// Table name
    pub name: String,
    /// List of column's info
    pub columns: Vec<ColumnInfo>,
    /// DDL of table
    pub create: String,
}

/// database description
pub struct DBInfo {
    /// database name
    pub name: String,
    /// DDL
    pub create: String,
    /// tables in this database
    pub tables: Vec<TableInfo>,
}

/// mysql connection info
pub struct DBOpt {
    /// username of mysql
    pub user: String,
    /// password of mysql
    pub password: String,
    /// host/IP to connect to mysql server
    pub host: String,
    /// port number to connect to mysql server
    pub port: i32,
    /// database name to connect to
    pub database: String,
}
