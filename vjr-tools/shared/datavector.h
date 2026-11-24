#ifndef DATAVECTOR
#define DATAVECTOR

#include <stdlib.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>


/*
Struct for dynamically storing and resizing a character array.
*/
typedef struct dataVector {
    char* data;
    uint32_t capacity;
    uint32_t index;
} dataVector;


/*
Initialises a dataVector.
@arg capacity -> initial capacity to give.
@return new dataVector.
*/
dataVector dataVectorInit(uint32_t capacity) {
    if (capacity == 0) {
        return (dataVector){0, 0, 0};
    }
    
    dataVector target = {
        .data = (char*) calloc(capacity, sizeof(char)),
        .capacity = capacity,
        .index = 0,
    };

    return target;
}

/*
Pushes a character onto a dataVector.
@arg target -> dataVector to push character onto.
@arg c -> character to push.
@return status of push.
*/
bool dataVectorPush(dataVector* target, char c) {
    uint32_t load = (target->index  * 100) / target->capacity;
    if (load > 60) {
        target->capacity <<= 1;
        target->data = (char*) realloc(target->data, target->capacity * sizeof(char));
        if (!target->data) return false;
    }

    target->data[target->index++] = c;
    target->data[target->index] = 0; //null terminate it.
    return true;
}

/*
Pushes characters from string onto a dataVector.
@arg target -> dataVector to push characters onto.
@arg s -> string to push.
@return status of push
*/
bool dataVectorPushString(dataVector* target, const char* s) {
    if (!s) return false;
    size_t size = strlen(s);
    for (size_t i = 0; i < size; i++) {
        if (!dataVectorPush(target, s[i])) return false;
    }
    return true;
}

/*
Pushes number of bytes onto a dataVector.
@arg target -> dataVector to push characters onto.
@arg b -> bytes to push.
@arg count -> number of bytes.
@return status of push.
*/
bool dataVectorPushBytes(dataVector* target, const char* b, size_t count) {
    if (!b) return false;
    for (size_t i = 0; i < count; i++) {
        if (!dataVectorPush(target, b[i])) return false;
    }
    return true;
}

#endif
