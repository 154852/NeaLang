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

void nl_drop_object(void* object, size_t size) {
	free(object);
}

void nl_drop_slice(struct slice_t* slice, size_t element_size) {
	free(slice);
}