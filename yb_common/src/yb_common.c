#include "yb_common.h"

#include <assert.h>
#include <malloc.h>
#include <pthread.h>
#include <stdio.h>
#include <string.h>

#ifndef YB_STRING_FLAG_OWN_DATA
/// \brief bitmask for yb_string_s::flags. If set, the yb_string_s owns the
/// data, and should free it when the yb_string_s is freed.
#define YB_STRING_FLAG_OWN_DATA 1
#endif

/**
 * @brief define a string type.
 */
struct yb_string_s {
    //! The string data, owned if flag & YB_STRING_FLAG_OWN_DATA is set, or
    //! reference if not.
    const char* data;
    //! The length of the string.
    int64_t len;
    //! The flags of the string.
    //! @see YB_STRING_FLAG_XXXXX
    uint32_t flag;
};

//! @brief the size of struct yb_string_s
#define YB_STRUCT_SIZE sizeof(struct yb_string_s)

yb_string_t yb_string_new() {
    yb_string_t s = malloc(YB_STRUCT_SIZE);
    s->data = NULL;
    s->len = 0;
    s->flag = 0;
    return s;
}

void yb_string_clear(yb_string_t s) {
    if (s == NULL) {
        return;
    }
    if (s->flag & YB_STRING_FLAG_OWN_DATA && s->data != NULL) {
        free((void*)s->data);
    }
    s->data = NULL;
    s->len = 0;
    s->flag = 0;
}

yb_string_t yb_string_from_ref(const char* str, int64_t len) {
    yb_string_t s = (struct yb_string_s*)malloc(YB_STRUCT_SIZE);
    if (s == NULL) {
        return s;
    }
    s->data = str;
    s->len = len;
    s->flag = 0;
    return s;
}

yb_string_t yb_string_from(const char* str, int64_t len) {
    char* copy = (char*)malloc(len);
    if (copy == NULL) {
        return NULL;
    }
    memcpy(copy, str, len);
    yb_string_t r = yb_string_from_ref(copy, len);
    if (r == NULL) {
        free(copy);
    }
    r->flag |= YB_STRING_FLAG_OWN_DATA;
    return r;
}

void yb_string_free(yb_string_t s) {
    if (s == NULL) {
        return;
    }
    if (s->flag & YB_STRING_FLAG_OWN_DATA && s->data != NULL) {
        free((void*)s->data);
    }
    free(s);
}

yb_string_t yb_string_from_cstr(const char* str) {
    return yb_string_new(str, strlen(str));
}

yb_string_t yb_string_from_cstr_ref(const char* str) {
    return yb_string_from_ref(str, strlen(str));
}

yb_string_t yb_string_clone(const yb_string_t s) {
    if (s == NULL) {
        return NULL;
    }
    return yb_string_new(s->data, s->len);
}

void yb_string_move(yb_string_t s, yb_string_t d) {
    if (s == NULL || d == NULL) {
        return;
    }

    d->data = s->data;
    d->len = s->len;
    d->flag = s->flag;
    s->data = NULL;
    s->len = 0;
    s->flag = 0;
}

const char* yb_string_cstr(const yb_string_t s) {
    if (s == NULL || s->len == 0) {
        return NULL;
    }

    char* r = (char*)malloc(s->len + 1);
    if (r == NULL) {
        return NULL;
    }
    memcpy(r, s->data, s->len);
    r[s->len] = '\0';
    return r;
}

int64_t yb_string_length(const yb_string_t s) {
    if (s == NULL) {
        return 0;
    }
    return s->len;
}

const char* yb_string_data(const yb_string_t s) {
    if (s == NULL) {
        return NULL;
    }
    return s->data;
}

int yb_string_set_data_from(yb_string_t s, const char* data, int64_t len) {
    if (s == NULL) {
        return YB_FAIL;
    }

    char* p = (char*)malloc(len);
    if (p == NULL) {
        return YB_FAIL;
    }
    memcpy(p, data, len);

    if (s->flag & YB_STRING_FLAG_OWN_DATA && s->data != NULL) {
        free((void*)s->data);
    }
    s->data = p;
    s->len = len;
    s->flag |= YB_STRING_FLAG_OWN_DATA;

    return YB_OK;
}

int yb_string_ref_data(yb_string_t s, const char* data, int64_t len) {
    if (s == NULL) {
        return YB_FAIL;
    }
    if (s->flag & YB_STRING_FLAG_OWN_DATA && s->data != NULL) {
        free((void*)s->data);
    }
    s->data = data;
    s->len = len;
    s->flag &= ~YB_STRING_FLAG_OWN_DATA;
    return YB_OK;
}

int yb_string_append_data(yb_string_t s, const char* data, int64_t len) {
    if (len == 0) {
        return YB_OK;
    }
    if (s == NULL) {
        return YB_FAIL;
    }
    char* p = (char*)malloc(s->len + len);
    if (p == NULL) {
        return YB_FAIL;
    }
    memcpy(p, s->data, s->len);
    memcpy(p + s->len, data, len);
    if (s->flag & YB_STRING_FLAG_OWN_DATA && s->data != NULL) {
        free((void*)s->data);
    }
    s->data = p;
    s->len += len;
    s->flag |= YB_STRING_FLAG_OWN_DATA;
    return YB_OK;
}

int yb_string_append_c_str(yb_string_t s, const char* str) {
    return yb_string_append_data(s, str, strlen(str));
}

int yb_string_append(yb_string_t s, const yb_string_t s2) {
    if (s == NULL || s2 == NULL) {
        return YB_FAIL;
    }
    return yb_string_append_data(s, s2->data, s2->len);
}

yb_string_t yb_string_substr(const yb_string_t s, int64_t start, int64_t len) {
    if (s == NULL) {
        return NULL;
    }
    if (start < 0 || len < 0 || start + len > s->len) {
        return NULL;
    }
    char* p = (char*)malloc(len);
    if (p == NULL) {
        return NULL;
    }
    memcpy(p, s->data + start, len);
    yb_string_t r = yb_string_from_ref(p, len);
    if (r == NULL) {
        free(p);
    }
    r->flag |= YB_STRING_FLAG_OWN_DATA;
    return r;
}

yb_string_t yb_string_slice(const yb_string_t s, int64_t start, int64_t len) {
    if (s == NULL) {
        return NULL;
    }
    if (start < 0 || len < 0 || start + len > s->len) {
        return NULL;
    }
    yb_string_t r = yb_string_from_ref(s->data + start, len);
    if (r == NULL) {
        return NULL;
    }
    r->flag = 0;
    return r;
}

int yb_string_compare(const yb_string_t s1, const yb_string_t s2) {
    if (s1 == NULL && s2 == NULL) {
        return 0;
    }

    if (s1 == NULL) {
        return -1;
    }
    if (s2 == NULL) {
        return 1;
    }

    int64_t min_len = s1->len < s2->len ? s1->len : s2->len;
    int cmp = memcmp(s1->data, s2->data, min_len);

    if (cmp == 0) {
        if (s1->len < s2->len) {
            return -1;
        } else if (s1->len > s2->len) {
            return 1;
        }
    }
    return cmp;
}

int yb_string_compare_cstr(const yb_string_t s1, const char* s2) {
    if (s1 == NULL && s2 == NULL) {
        return 0;
    }
    if (s1 == NULL) {
        return -1;
    }
    if (s2 == NULL) {
        return 1;
    }
    int64_t s2len = strlen(s2);
    int64_t min_len = s1->len < s2len ? s1->len : s2len;
    int cmp = memcmp(s1->data, s2, min_len);
    if (cmp == 0) {
        if (s1->len < s2len) {
            return -1;
        } else if (s1->len > s2len) {
            return 1;
        }
    }
    return cmp;
}

int64_t yb_string_atoi(const yb_string_t s) {
    if (s == NULL) {
        return 0;
    }
    char* end = (char*)s->data;
    int64_t r = 0;

    while (end != s->data + s->len) {
        if (*end < '0' || *end > '9') {
            break;
        }
        r = r * 10 + *end - '0';
        end++;
    }
    return r;
}

int yb_string_to_hex(const yb_string_t s, char* out, int64_t out_len) {
    if (s == NULL) {
        return YB_FAIL;
    }
    if (out == NULL) {
        return YB_FAIL;
    }

    int64_t i;
    int64_t j = 0;
    for (i = 0; i < s->len; i++) {
        if (j + 3 >= out_len) {
            break;
        }
        const uint8_t c = (uint8_t)s->data[i];
        snprintf(out + j, 3, "%02x", c);
        j += 2;
    }
    out[j] = '\0';
    return YB_OK;
}

static inline int _char2int(const char c) {
    if (c >= '0' && c <= '9') {
        return c - '0';
    } else if (c >= 'a' && c <= 'f') {
        return c - 'a' + 10;
    } else if (c >= 'A' && c <= 'F') {
        return c - 'A' + 10;
    }
    return -1;
}

int yb_string_from_hex(yb_string_t s, const char* in, int64_t in_len) {
    if (s == NULL) {
        return YB_FAIL;
    }
    if (in == NULL) {
        yb_string_clear(s);
        return YB_OK;
    }

    int64_t plen = (in_len + 1) / 2;
    char* p = (char*)malloc(plen);
    if (p == NULL) {
        return YB_FAIL;
    }

    int64_t i = 0;
    int64_t j = 0;
    if (in_len % 2 != 0) {
        int d = _char2int(in[i]);
        if (d < 0) {
            free(p);
            return YB_FAIL;
        }
        p[j] = in[i];
        ++i;
        ++j;
    }

    while (i < in_len) {
        int d1 = _char2int(in[i]);
        int d2 = _char2int(in[i + 1]);
        if (d1 < 0 || d2 < 0) {
            free(p);
            return YB_FAIL;
        }
        p[j] = (char)((d1 << 4) | d2);
        i += 2;
        ++j;
    }
    yb_string_ref_data(s, p, plen);
    s->flag |= YB_STRING_FLAG_OWN_DATA;
    return YB_OK;
}

int yb_string_trim(yb_string_t src, const char* prefix, const char* suffix,
                   const char* prefix_override, const char* suffix_override,
                   yb_string_t dest) {
    if (src == NULL) {
        return YB_FAIL;
    }
    if (prefix == NULL || suffix == NULL) {
        return YB_FAIL;
    }

    int64_t start = 0;
    int64_t end = src->len;
    int64_t prefix_len = strlen(prefix_override);
    int64_t suffix_len = strlen(suffix_override);
    if (prefix_len + suffix_len > src->len) {
        return YB_FAIL;
    }
    yb_string_t to_remove = yb_string_new();
    yb_string_ref_data(to_remove, src->data + prefix_len, prefix_len);
    if (yb_string_compare(to_remove, prefix_override) == 0) {
        start = prefix_len;
    }
    yb_string_ref_data(to_remove, src->data + src->len - suffix_len,
                       suffix_len);
    if (yb_string_compare(to_remove, suffix_override) == 0) {
        end -= suffix_len;
    }

    yb_string_free(to_remove);

    yb_string_clear(dest);
    yb_string_append_c_str(dest, prefix);
    yb_string_append_data(dest, src->data + start, end - start);
    yb_string_append_c_str(dest, suffix);

    return YB_OK;
}
static inline uint32_t murmurhash(const char* key, uint32_t len,
                                  uint32_t seed) {
    uint32_t c1 = 0xcc9e2d51;
    uint32_t c2 = 0x1b873593;
    uint32_t r1 = 15;
    uint32_t r2 = 13;
    uint32_t m = 5;
    uint32_t n = 0xe6546b64;
    uint32_t h = 0;
    uint32_t k = 0;
    uint8_t* d = (uint8_t*)key;  // 32 bit extract from `key'
    const uint32_t* chunks = NULL;
    const uint8_t* tail = NULL;  // tail - last 8 bytes
    int i = 0;
    int l = len / 4;  // chunk length

    h = seed;

    chunks = (const uint32_t*)(d + l * 4);  // body
    tail = (const uint8_t*)(d + l * 4);     // last 8 byte chunk of `key'

    // for each 4 byte chunk of `key'
    for (i = -l; i != 0; ++i) {
        // next 4 byte chunk of `key'
        k = chunks[i];

        // encode next 4 byte chunk of `key'
        k *= c1;
        k = (k << r1) | (k >> (32 - r1));
        k *= c2;

        // append to hash
        h ^= k;
        h = (h << r2) | (h >> (32 - r2));
        h = h * m + n;
    }

    k = 0;

    // remainder
    switch (len & 3) {  // `len % 4'
        case 3:
            k ^= (tail[2] << 16);
            __attribute__((fallthrough));
        case 2:
            k ^= (tail[1] << 8);
            __attribute__((fallthrough));

        case 1:
            k ^= tail[0];
            k *= c1;
            k = (k << r1) | (k >> (32 - r1));
            k *= c2;
            h ^= k;
    }

    h ^= len;

    h ^= (h >> 16);
    h *= 0x85ebca6b;
    h ^= (h >> 13);
    h *= 0xc2b2ae35;
    h ^= (h >> 16);

    return h;
}

struct yb_hash_entry {
    yb_string_t key;
    yb_string_t value;
    uint32_t hash;
    struct yb_hash_entry* next;
};

struct yb_hash_map_s {
    // array of bucket pointer to linked list of yb_hash_entry
    struct yb_hash_entry** head;
    // length of head
    int32_t head_length;
    // number of elements in hash map
    int32_t elems;
};

static struct yb_hash_entry* yb_hash_entry_new(yb_string_t key,
                                               yb_string_t value,
                                               int32_t hash) {
    struct yb_hash_entry* n =
        (struct yb_hash_entry*)malloc(sizeof(struct yb_hash_entry));
    if (n == NULL) {
        return NULL;
    }
    n->key = yb_string_clone(key);
    if (n->key == NULL) {
        free(n);
        return NULL;
    }
    n->value = yb_string_clone(value);
    if (n->value == NULL) {
        yb_string_free(n->key);
        free(n);
        return NULL;
    }
    n->hash = hash;
    n->next = NULL;
    return n;
}

static void yb_hash_entry_free(struct yb_hash_entry* e) {
    if (e == NULL) {
        return;
    }
    yb_string_free(e->key);
    yb_string_free(e->value);
    free(e);
}

// expected map->head_length ~= map->elems.
// map->head_length is always a power of 2.
static int yb_hash_map_resize(yb_hash_map_t map) {
    int32_t new_length = 4;
    while (new_length < map->elems) {
        new_length *= 2;
    }

    struct yb_hash_entry** new_head = (struct yb_hash_entry**)malloc(
        new_length * sizeof(struct yb_hash_entry*));
    if (new_head == NULL) {
        return YB_FAIL;
    }
    memset(new_head, 0, new_length * sizeof(struct yb_hash_entry*));

    int count = 0;
    for (int32_t i = 0; i < map->head_length; i++) {
        struct yb_hash_entry* h = map->head[i];
        while (h != NULL) {
            struct yb_hash_entry* next = h->next;
            int32_t hash = h->hash;
            struct yb_hash_entry** ptr = &new_head[hash & (new_length - 1)];
            h->next = *ptr;
            *ptr = h;
            h = next;
            count++;
        }
    }
    assert(map->elems == count);
    free(map->head);  // if free(NULL), no action occurs.
    map->head = new_head;
    map->head_length = new_length;
    return YB_OK;
}

yb_hash_map_t yb_hash_map_new() {
    yb_hash_map_t map = (yb_hash_map_t)malloc(sizeof(struct yb_hash_map_s));
    if (map == NULL) {
        return NULL;
    }
    map->head = NULL;
    map->head_length = 0;
    map->elems = 0;
    int r = yb_hash_map_resize(map);
    if (r != YB_OK) {
        free(map);
        return NULL;
    }
    return map;
}

void yb_hash_map_free(yb_hash_map_t map) {
    if (map == NULL) {
        return;
    }

    for (int32_t i = 0; i < map->head_length; i++) {
        struct yb_hash_entry* h = map->head[i];
        while (h != NULL) {
            struct yb_hash_entry* next = h->next;
            yb_string_free(h->key);
            yb_string_free(h->value);
            free(h);
            h = next;
        }
    }
    free(map->head);
    free(map);
}

// find the position of `key' in the hash map.
static inline struct yb_hash_entry** yb_hash_map_find_pointer(yb_hash_map_t map,
                                                              yb_string_t key,
                                                              uint32_t hash) {
    struct yb_hash_entry** ptr = &map->head[hash & (map->head_length - 1)];
    while (*ptr != NULL &&
           ((*ptr)->hash != hash || yb_string_compare(key, (*ptr)->key) != 0)) {
        ptr = &(*ptr)->next;
    }
    return ptr;
}

int yb_hash_map_insert(yb_hash_map_t map, yb_string_t key, yb_string_t value) {
    if (map == NULL) {
        return YB_FAIL;
    }
    if (key == NULL) {
        return YB_FAIL;
    }

    int32_t hash = murmurhash(key->data, key->len, 0);
    struct yb_hash_entry** ptr = yb_hash_map_find_pointer(map, key, hash);
    struct yb_hash_entry* old = *ptr;

    struct yb_hash_entry* h = yb_hash_entry_new(key, value, hash);
    if (h == NULL) {
        return YB_FAIL;
    }
    h->next = (old == NULL) ? NULL : old->next;
    *ptr = h;
    yb_hash_entry_free(old);

    if (old == NULL) {
        map->elems++;
        if (map->elems > map->head_length) {
            yb_hash_map_resize(map);
        }
    }
    return YB_OK;
}

yb_string_t yb_hash_map_get(yb_hash_map_t map, yb_string_t key) {
    if (map == NULL) {
        return NULL;
    }
    if (key == NULL) {
        return NULL;
    }

    int32_t hash = murmurhash(key->data, key->len, 0);
    struct yb_hash_entry** ptr = yb_hash_map_find_pointer(map, key, hash);
    if (*ptr == NULL) {
        return NULL;
    }
    return (*ptr)->value;
}

void yb_hash_map_remove(yb_hash_map_t map, yb_string_t key) {
    if (map == NULL) {
        return;
    }
    if (key == NULL) {
        return;
    }
    int32_t hash = murmurhash(key->data, key->len, 0);
    struct yb_hash_entry** ptr = yb_hash_map_find_pointer(map, key, hash);
    struct yb_hash_entry* h = *ptr;
    if (h == NULL) {
        return;
    }
    *ptr = h->next;
    yb_hash_entry_free(h);
    map->elems--;
}

#define TEST_CASE_W(cond) arg->##cond
#define yb_stmt_gen_string(str, e, arg_t, arg)           \
    do {                                                 \
        struct yb_stmt_elem* cur = e;                    \
        for (; cur; cur = cur->next) {                   \
            switch (cur->type) {                         \
                case YB_STMT_ELEM_TYPE_TEXT:             \
                    yb_string_append(s, cur->text.text); \
                    break;                               \
                case YB_STMT_ELEM_TYPE_IF:               \
            }                                            \
        }                                                \
    } while (0)

yb_string_t yb_stmt_gen_string(struct yb_stmt_elem* e, void* arg) {
    yb_string_t s = yb_string_new();

    struct yb_stmt_elem* cur = e;
    for (; cur; cur = cur->next) {
        switch (cur->type) {
            case YB_STMT_ELEM_TYPE_TEXT: {
                yb_string_append(s, cur->text.text);
                break;
            }
            case YB_STMT_ELEM_TYPE_IF: {
                // TODO
                break;
            }
            case YB_STMT_ELEM_TYPE_TRIM: {
                // TODO
                break;
            }
        }
    }

    return s;
}

// mysql connection pool entry
struct yb_mysql_entry_s {
    MYSQL* conn;
    struct yb_mysql_entry_s* next;
    time_t touch;
};
// mysql connection pool
struct yb_mysql_pool_s {
    struct yb_mysql_entry_s* avail;
    struct yb_mysql_entry_s* busy;
    int max_connections;
    int min_connections;
    int cur_connections;
    int64_t max_idle_time;
    int64_t last_check_time;
    int64_t last_check_idle_time;

    const char* host;
    int port;
    const char* user;
    const char* passwd;
    const char* dbname;
    const char* charset;
    int connect_timeout;
    int read_timeout;
    int write_timeout;
    int max_retries;

    pthread_mutex_t mutex;
};

yb_mysql_pool_t yb_mysql_pool_new(const char* host, int port, const char* user,
                                  const char* passwd, const char* dbname) {
    yb_mysql_pool_t pool =
        (yb_mysql_pool_t)malloc(sizeof(struct yb_mysql_pool_s));
    if (pool == NULL) {
        return NULL;
    }
    pool->avail = NULL;
    pool->busy = NULL;
    pool->max_connections = 10;
    pool->min_connections = 5;
    pool->cur_connections = 0;
    pool->max_idle_time = 60 * 1000;
    pool->last_check_time = 0;
    pool->last_check_idle_time = 0;
    pool->host = host;
    pool->port = port;
    pool->user = user;
    pool->passwd = passwd;
    pool->dbname = dbname;
    pool->charset = "utf8";
    pool->connect_timeout = 3;
    pool->read_timeout = 60;
    pool->write_timeout = 60;
    pool->max_retries = 3;
    pthread_mutex_init(&pool->mutex, NULL);
    return pool;
}

#define __POOL_LOCK(pool) pthread_mutex_lock(&pool->mutex)
#define __POOL_UNLOCK(pool) pthread_mutex_unlock(&pool->mutex)

void yb_mysql_pool_option_max_connections(yb_mysql_pool_t pool, int max) {
    __POOL_LOCK(pool);
    pool->max_connections = max;
    __POOL_UNLOCK(pool);
}
void yb_mysql_pool_option_min_connections(yb_mysql_pool_t pool, int min) {
    __POOL_LOCK(pool);
    pool->min_connections = min;
    __POOL_UNLOCK(pool);
}
void yb_mysql_pool_option_max_idle_time(yb_mysql_pool_t pool, int64_t seconds) {
    __POOL_LOCK(pool);
    pool->max_idle_time = seconds;
    __POOL_UNLOCK(pool);
}

void yb_mysql_pool_option_connect_timeout(yb_mysql_pool_t pool, int timeout) {
    __POOL_LOCK(pool);
    pool->connect_timeout = timeout;
    __POOL_UNLOCK(pool);
}

void yb_mysql_pool_option_read_timeout(yb_mysql_pool_t pool, int timeout) {
    __POOL_LOCK(pool);
    pool->read_timeout = timeout;
    __POOL_UNLOCK(pool);
}

void yb_mysql_pool_option_write_timeout(yb_mysql_pool_t pool, int timeout) {
    __POOL_LOCK(pool);
    pool->write_timeout = timeout;
    __POOL_UNLOCK(pool);
}

void yb_mysql_pool_option_max_retries(yb_mysql_pool_t pool, int max) {
    __POOL_LOCK(pool);
    pool->max_retries = max;
    __POOL_UNLOCK(pool);
}

void yb_mysql_pool_option_charset(yb_mysql_pool_t pool, const char* charset) {
    __POOL_LOCK(pool);
    pool->charset = charset;
    __POOL_UNLOCK(pool);
}

static MYSQL* yb_mysql_try_new_conn(yb_mysql_pool_t pool) {
    MYSQL* conn = mysql_init(NULL);
    if (conn == NULL) {
        return NULL;
    }
    if (mysql_real_connect(conn, pool->host, pool->user, pool->passwd,
                           pool->dbname, pool->port, NULL, 0) == NULL) {
        mysql_close(conn);
        return NULL;
    }
    if (mysql_set_character_set(conn, pool->charset) != 0) {
        mysql_close(conn);
        return NULL;
    }
    if (mysql_options(conn, MYSQL_OPT_CONNECT_TIMEOUT,
                      &pool->connect_timeout) != 0) {
        mysql_close(conn);
        return NULL;
    }
    if (mysql_options(conn, MYSQL_OPT_READ_TIMEOUT, &pool->read_timeout) != 0) {
        mysql_close(conn);
        return NULL;
    }
    if (mysql_options(conn, MYSQL_OPT_WRITE_TIMEOUT, &pool->write_timeout) !=
        0) {
        mysql_close(conn);
        return NULL;
    }

    return conn;
}

MYSQL* yb_mysql_pool_get_connection(yb_mysql_pool_t pool) {
    __POOL_LOCK(pool);
    MYSQL* conn = NULL;
    struct yb_mysql_entry_s* entry = NULL;
    time_t cur = time(NULL);

    while (conn == NULL && pool->avail != NULL) {
        entry = pool->avail;
        pool->avail = entry->next;
        // clean edle timeout connections
        if (cur > pool->max_idle_time + entry->touch) {
            mysql_close(entry->conn);
            free(entry);
            entry = NULL;
            pool->cur_connections--;
            continue;
        }
    }
    if (entry != NULL) {
        entry->next = pool->busy;
        pool->busy = entry;
        entry->touch = cur;
        __POOL_UNLOCK(pool);
        return entry->conn;
    }

    int retrys = pool->max_retries;
    while (conn == NULL && pool->cur_connections < pool->max_connections &&
           retrys-- > 0) {
        conn = yb_mysql_try_new_conn(pool);
        if (conn) {
            pool->cur_connections++;
            struct yb_mysql_entry_s* entry = (struct yb_mysql_entry_s*)malloc(
                sizeof(struct yb_mysql_entry_s));
            entry->conn = conn;
            entry->next = pool->busy;
            entry->touch = cur;
            pool->busy = entry;
        }
    }
    __POOL_UNLOCK(pool);
    return conn;
}

static void yb_mysql_pool_prune(yb_mysql_pool_t pool) {
    while (pool->cur_connections > pool->min_connections) {
        struct yb_mysql_entry_s* entry = pool->avail;
        pool->avail = entry->next;
        mysql_close(entry->conn);
        free(entry);
        pool->cur_connections--;
    }
}

void yb_mysql_pool_release_connection(yb_mysql_pool_t pool, MYSQL* conn) {
    __POOL_LOCK(pool);
    assert(pool->busy != NULL);

    struct yb_mysql_entry_s* entry = pool->busy;
    pool->busy = entry->next;
    entry->next = pool->avail;
    pool->avail = entry;
    entry->conn = conn;
    yb_mysql_pool_prune(pool);
    __POOL_UNLOCK(pool);
}
