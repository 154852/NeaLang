#include <stdlib.h>
#include <stdio.h>

struct __attribute__((packed)) slice_t {
	void* data;
	size_t len;
};

struct slice_t* nl_alloc_slice_u8(size_t length) {
	struct slice_t* slice = malloc(sizeof(struct slice_t) + length);
	slice->data = slice + sizeof(struct slice_t);
	slice->len = length;
	return slice;
}

struct slice_t* nl_alloc_slice_u32(size_t length) {
	struct slice_t* slice = malloc(sizeof(struct slice_t) + (length * sizeof(unsigned int)));
	slice->data = slice + sizeof(struct slice_t);
	slice->len = length;
	return slice;
}