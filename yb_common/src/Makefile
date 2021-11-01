.PHONY: libyb_common

LIB_SYS = mysqlclient

CFLAGS += -Wall -Wextra -Werror -Wundef -Wvla -pedantic `pkg-config --cflags $(LIB_SYS)`
LDFLAGS += `pkg-config --libs $(LIB_SYS)`

ifeq ($(TARGET_DIR),)
TARGET_DIR = $(PWD)/target
endif

$(shell mkdir -p $(TARGET_DIR))

sources = $(wildcard *.c)
headers = $(wildcard *.h)
objs = $(patsubst %.c,$(TARGET_DIR)/%.o,$(sources))

libyb_common: libyb_common.a

libyb_common.a: $(objs)
	ar rcs $(TARGET_DIR)/$@ $(LDFLAGS) $^

$(objs): $(sources)
	$(CC) $(CFLAGS)-c -o $@ $<

