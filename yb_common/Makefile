
.PHONY: all libyb_common target_folder clean

MKFILE_PATH = $(abspath $(lastword $(MAKEFILE_LIST)))
MAKEFILE_DIR = $(dir $(MKFILE_PATH))
TARGET_DIR = $(MAKEFILE_DIR)target/

export TARGET_DIR

all: libyb_common	

target_folder:
	mkdir -p $(TARGET_DIR)

libyb_common:
	$(MAKE) -C src $@

clean:
	rm -rf $(TARGET_DIR)
