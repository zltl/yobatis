#ifndef YB_COMMON_H__
#define YB_COMMON_H__

#include <float.h>
#include <mysql/mysql.h>
#include <stdint.h>

#ifndef YB_INT_NULL
/// Null value for integer (int64_t)
#define YB_INT_NULL INT64_MIN
#endif

#ifndef YB_FLOAT_NULL
/// Null value for float (double)
#define YB_FLOAT_NULL DBL_MIN
#endif

#ifndef YB_STRING_NULL
/// Null value for string (yb_string_s)
#define YB_STRING_NULL NULL
#endif

#ifndef YB_OK
/// Success value
#define YB_OK 0
#endif
#ifndef YB_FAIL
/// Fail value
#define YB_FAIL -1
#endif

/// string type yb_string_t generalizes how sequences of bytes are manipulated
/// and stored. String creation, manipulation, and description are all handled
/// by a convenient set of functions.
struct yb_string_s;
typedef struct yb_string_s* yb_string_t;

/// @brief Creates a new empty yb_string_t object.
/// @note the returned object must be freed with yb_string_free()
yb_string_t yb_string_new();

/// @brief make \a s to be an empty string.
void yb_string_clear(yb_string_t s);

/// @brief Frees an yb_string_t object.
void yb_string_free(yb_string_t s);

/// @brief create an yb_string_t object and copy contents with length of
/// \a len from \a str to new yb_string_t object. The newd object should free
/// by yb_string_free().
///
/// @param str Pointer to contents to copy.
/// @param len Length of contents to copy.
/// @return new yb_string_t object, with contents copied from \a str.
/// @retval NULL Memory allocation failed.
yb_string_t yb_string_from(const char* str, int64_t len);

/// @brief create an yb_string_t object with reference to \a str. The
/// newd object should free by yb_string_free(). The memory allocated for \a
/// str is not copied, so it will not be freed by yb_string_free(). \a str
/// should not be freed before the newd object.
///
/// @param str Pointer to contents to reference.
/// @param len Length of contents to reference.
/// @return new yb_string_t object, with contents referenced from \a
/// str.
/// @retval NULL Memory allocation failed.
/// @note The string reference is not copied, so it should not be freed
/// before the newd object.
yb_string_t yb_string_from_ref(const char* str, int64_t len);

/// @brief create an yb_string_t object with content copied from \a str.
/// The newd object should free by yb_string_free().
///
/// @param str a c-string to copy.
/// @return new yb_string_t object, with contents copied from \a str.
/// @retval NULL Memory allocation failed.
yb_string_t yb_string_from_cstr(const char* str);

/// @brief create an yb_string_t object with refence to the content of \a
/// str.
///
/// @param str a c-string to reference.
/// @return new yb_string_t object, with contents referenced from \a str.
/// @retval NULL Memory allocation failed.
/// @note The string reference is not copied, so it should not be freed
/// before the newd object.
yb_string_t yb_string_from_cstr_ref(const char* str);

/// @brief create an yb_string_t object and copy contents from \a s.
/// The newd object should free by yb_string_free().
///
/// @param s a yb_string_t object to copy.
/// @return snew yb_string_t object, with contents copied from \a s.
/// @retval NULL Memory allocation failed.
yb_string_t yb_string_clone(const yb_string_t s);

/// @brief move the contents of \a s to \a d.
/// \a s will be empty after this call.
///
/// @param s a yb_string_t object to move from.
/// @param d a yb_string_t object to move to.
/// @note \a s will be an empty yb_string_t object after this call. \a s should
/// be free by yb_string_free() though.
void yb_string_move(yb_string_t s, yb_string_t d);

/// @brief create and return a c-string null-terminated buffer with the contents
/// of \a s.
///
/// @param s a yb_string_t
/// @return a c-string buffer with the contents of \a s.
/// @note The returned buffer should be freed by the caller.
const char* yb_string_cstr(const yb_string_t s);

/// @brief return the content's length of \a s.
/// @param s a yb_string_t
/// @return the content's length of \a s. 0 if s==NULL.
int64_t yb_string_length(const yb_string_t s);

/// @brief return the pointer the the content of pointer of \a s.
/// @param s an yb_string_t
/// @return the pointer to the content of \a s. NULL if s==NULL.
const char* yb_string_data(const yb_string_t s);

/// @brief clone(copy) the contents from \a data with lengths of \a len to \a s.
/// The old contents of \a s will be freed if owned by \a s.
///
/// @param s an yb_string_t
/// @param data a pointer to the content to copy.
/// @param len the length of the content to copy.
/// @retval YB_OK if the operation is successful.
/// @retval YB_FAIL if the operation is failed.
int yb_string_set_data_from(yb_string_t s, const char* data, int64_t len);

/// @brief make \a s reference to \a data with lengths of \a len.
/// The old contents of \a s will be freed if owned by \a s.
/// \a data should not be freed before \a s.
///
/// @param s an yb_string_t
/// @param data a pointer to the content to reference.
/// @param len the length of the content to reference.
/// @retval YB_OK if the operation is successful.
/// @retval YB_FAIL if the operation is failed.
/// @note The \a data should not be freed before \a s.
int yb_string_ref_data(yb_string_t s, const char* data, int64_t len);

/// @brief append a copy of \a data with length of \a len to \a s.
///
/// @param s an yb_string_t
/// @param data a pointer to the content to append.
/// @param len the length of the content to append.
/// @retval YB_OK if the operation is successful.
/// @retval YB_FAIL if the operation is failed.
int yb_string_append_data(yb_string_t s, const char* data, int64_t len);

/// @brief append a copy of \a str with length of \a strlen(str) to \a s.
///
/// @param s an yb_string_t
/// @param str a pointer c-string.
/// @retval YB_OK if the operation is successful.
/// @retval YB_FAIL if the operation is failed.
int yb_string_append_c_str(yb_string_t s, const char* str);

/// @brief append a copy of \a s2 to \a s.
///
/// @param s an yb_string_t
/// @param s2 an yb_string_t to append.
/// @retval YB_OK if the operation is successful.
/// @retval YB_FAIL if the operation is failed.
int yb_string_append(yb_string_t s, const yb_string_t s2);

/// @brief create an yb_string_t object with a slice of copy content of \a s.
///
/// @param s an yb_string_t
/// @param start the start index of the slice.
/// @param len the length of the slice.
/// @return a new yb_string_t object with a slice of copy content of \a s.
/// @retval NULL Memory allocation failed.
yb_string_t yb_string_substr(const yb_string_t s, int64_t start, int64_t len);

/// @brief create an yb_string_t object with a slice of reference content of \a
/// s. \a s should not be freed before the newd object.
///
/// @param s an yb_string_t
/// @param start the start index of the slice.
/// @param len the length of the slice.
/// @return a new yb_string_t object with a slice of reference content of \a s.
/// @retval NULL Memory allocation failed.
/// @note The \a s should not be freed before the newd object.
yb_string_t yb_string_slice(const yb_string_t s, int64_t start, int64_t len);

/// @brief compaire \a s1 and \a s2.
///
/// @param s1 an yb_string_t
/// @param s2 an yb_string_t
/// @retval -1 if \a s1 < \a s2.
/// @retval 0 if \a s1 == \a s2.
/// @retval 1 if \a s1 > \a s2.
/// @retval 0 if \a s1 == NULL and \a s2 == NULL.
/// @note NULL object is considered as smallest.
int yb_string_compare(const yb_string_t s1, const yb_string_t s2);

/// @brief compare yb_string_t \a s1 and c-string(null-termintate) \a s2.
///
/// @param s1 an yb_string_t
/// @param s2 a c-string
/// @retval -1 if \a s1 < \a s2.
/// @retval 0 if \a s1 == \a s2.
/// @retval 1 if \a s1 > \a s2.
/// @retval 0 if \a s1 == NULL and \a s2 == NULL.
/// @note NULL object is considered as smallest.
int yb_string_compare_cstr(const yb_string_t s1, const char* s2);

/// @brief parse \a s as a signed integer.
/// @param s an yb_string_t
/// @return the parsed integer.
int64_t yb_string_atoi(const yb_string_t s);

/// @brief convert content bytes of \a s to hexadecimal c-string.
/// @param s an yb_string_t
/// @param out pointer to buffer of output hexadecimal string.
/// @param out_len the length of the output buffer \a out.
/// @retval YB_OK if the operation is successful.
/// @retval YB_FAIL if the operation is failed, fail if \a s or \a out is NULL.
int yb_string_to_hex(const yb_string_t s, char* out, int64_t out_len);

/// @brief parse hexadecimal c-string \a in as bytes and copy to \a s.
/// @param s an yb_string_t
/// @param in a hexadecimal c-string
/// @param in_len the length of the input string \a in.
/// @retval YB_OK if the operation is successful.
/// @retval YB_FAIL if the operation is failed, fail if \a s is NULL, or \a in
/// is not a valid hexadecimal string.
int yb_string_from_hex(yb_string_t s, const char* in, int64_t in_len);

/// @brief trim string src -> dest.
/// @param src[in] source yb_string_t object.
/// @param prefix[in] add prefix to dest.
/// @param suffix[in] add suffix to dest.
/// @param prefix_override[in] will remove prefix of src.
/// @param suffix_override[in] will remove suffix of src.
/// @param dest[out] destination yb_string_t object.
int yb_string_trim(yb_string_t src, const char* prefix, const char* suffix,
                   const char* prefix_override, const char* suffix_override,
                   yb_string_t dest);

/// hash map from yb_string_t to yb_string_t.
struct yb_hash_map_s;
typedef struct yb_hash_map_s* yb_hash_map_t;

/// @brief create a new hash map.
/// @return a new hash map object.
/// @retval NULL Memory allocation failed.
yb_hash_map_t yb_hash_map_new();

/// @brief free a hash map.
/// @param map a hash map object.
void yb_hash_map_free(yb_hash_map_t map);

/// @brief insert a key-value pair to a hash map. \a key and \a value are copied
/// into map.
/// @param map a hash map object.
/// @param key a key to insert.
/// @param value a value to insert.
/// @retval YB_OK if the operation is successful.
/// @retval YB_FAIL if the operation is failed, fail if \a map is NULL, or \a
/// key is NULL.
int yb_hash_map_insert(yb_hash_map_t map, yb_string_t key, yb_string_t value);

/// @brief search a value from a hash map by key.
/// @param map a hash map object.
/// @param key a key to search.
/// @retval NULL if the key is not found.
/// @retval an yb_string_t value if the key is found.
yb_string_t yb_hash_map_get(yb_hash_map_t map, yb_string_t key);

/// @brief remove a key-value pair from a hash map.
/// @param map a hash map object.
/// @param key a key to remove.
void yb_hash_map_remove(yb_hash_map_t map, yb_string_t key);

/// @brief generate statements by yb_stmt_elem list.
/// @param e a yb_stmt_elem list.
/// @return a new yb_string_t object.
yb_string_t yb_stmt_gen_string(struct yb_stmt_elem* e);

/// define a thread-safe MYSQL connection pool.
struct yb_mysql_pool_s;
typedef struct yb_mysql_pool_s* yb_mysql_pool_t;

/// @brief create a new mysql pool.
/// @param host the host of mysql server.
/// @param port the port of mysql server.
/// @param user the user of mysql server.
/// @param password the password of mysql server.
/// @param database the database of mysql server.
/// @return a new mysql pool object.
/// @retval NULL Memory allocation failed.
/// @note The \a host, \a port, \a user, \a password and \a database should not
/// be freed before the newd object.
yb_mysql_pool_t yb_mysql_pool_new(const char* host, int port, const char* user,
                                  const char* passwd, const char* dbname);

/// @brief set the max number of connections in the pool.
/// @param pool a mysql pool object.
/// @param max_conn the max number of connections.
void yb_mysql_pool_option_max_connections(yb_mysql_pool_t pool, int max);

/// @brief set the minimum number of connections in the pool.
/// @param pool an yb_mysql_pool_t pool object.
/// @param min the minimum number of connections in the pool.
/// If the number of connections in the pool is larger then \a min, the
/// yb_mysql_pool_release_connection() will free some connections.
void yb_mysql_pool_option_min_connections(yb_mysql_pool_t pool, int min);

/// @brief set the maximum idle time seconds of each connections in the pool.
/// Connections with idle time exceeding this value will be closed.
/// @param pool an yb_mysql_pool_t pool object.
/// @param seconds the maximum idle time seconds.
void yb_mysql_pool_option_max_idle_time(yb_mysql_pool_t pool, int64_t seconds);

/// @brief set the connection timeout seconds.
/// @param pool an yb_mysql_pool_t pool object.
/// @param timeout the connection timeout seconds.
void yb_mysql_pool_option_connect_timeout(yb_mysql_pool_t pool, int timeout);

/// @brief set the max read timeout of each MYSQL connections.
/// @param pool an yb_mysql_pool_t pool object.
/// @param timeout the max read timeout seconds.
void yb_mysql_pool_option_read_timeout(yb_mysql_pool_t pool, int timeout);

/// @brief set the max write timeout of each MYSQL connections.
/// @param pool an yb_mysql_pool_t pool object.
/// @param timeout the max write timeout seconds.
void yb_mysql_pool_option_write_timeout(yb_mysql_pool_t pool, int timeout);

/// @brief set the max number of retries of each MYSQL connections.
/// @param pool an yb_mysql_pool_t pool object.
/// @param retries the max number of retries.
void yb_mysql_pool_option_max_retries(yb_mysql_pool_t pool, int max);

/// @brief set the charset of MYSQL client.
/// @param pool an yb_mysql_pool_t pool object.
/// @param charset the charset of MYSQL client.
/// @note default charset is "utf8"
void yb_mysql_pool_option_charset(yb_mysql_pool_t pool, const char* charset);

/// @brief get a MYSQL connection from the pool.
/// @param pool an yb_mysql_pool_t pool object.
/// @return a MYSQL connection object.
/// @retval NULL if the pool is empty or the connection is failed.
/// @note The returned connection should be freed by
/// yb_mysql_pool_release_connection().
/// @note The returned connection is not thread-safe.
MYSQL* yb_mysql_pool_get_connection(yb_mysql_pool_t pool);

/// @brief release a MYSQL connection to the pool.
/// @param pool an yb_mysql_pool_t pool object.
/// @param conn a MYSQL connection object.
void yb_mysql_pool_release_connection(yb_mysql_pool_t pool, MYSQL* conn);

#endif  // YB_COMMON_H__
