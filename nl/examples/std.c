#include <stdlib.h>
#include <stdio.h>

struct __attribute__((packed)) slice_t {
	void* data;
	size_t len;
};

void* nl_new_object(size_t size) {
	return malloc(size);
}

struct slice_t* nl_new_slice(size_t length, size_t size) {
	struct slice_t* slice = malloc(sizeof(struct slice_t) + length*size);
	slice->data = slice + sizeof(struct slice_t);
	slice->len = length;
	return slice;
}