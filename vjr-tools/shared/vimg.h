#ifndef VIMG_GUARD
#define VIMG_GUARD


#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>

#include "datavector.h"





typedef struct Vimg {
    char* name;
    char* data;
    uint32_t data_size;
    bool valid;
} Vimg;


typedef struct Args {
    char* input_file;
    char* output_file;
} Args;

Args parse_args(int argc, char** argv) {
    Args args = { 0 };

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-i") == 0 && (i + 1) < argc) {
            i++;
            args.input_file = (char*) calloc(strlen(argv[i]), sizeof(char));
            strncpy(args.input_file, argv[i], strlen(argv[i]));
        }
        else if (strcmp(argv[i], "-o") == 0 && (i + 1) < argc) {
            i++;
            args.output_file = (char*) calloc(strlen(argv[i]), sizeof(char));
            strncpy(args.output_file, argv[i], strlen(argv[i]));
        }
    }

    return args;
}

uint16_t convert(uint8_t R, uint8_t G, uint8_t B) {
    //ratios between max screen RGB values and standard RGB values
    const float Rk = 0.122;
    const float Gk = 0.248;
    const float Bk = 0.122;
    
    //apply the ratios
    uint8_t Rtransform = R >> 3;
    uint8_t Gtransform = G >> 2;
    uint8_t Btransform = B >> 3;

    uint16_t result = 0;

    //type casting float -> int -> uint8_t, then bit OR into the result.

    int Rint = (int) Rtransform;
    uint8_t Rbyte = (uint8_t) Rint;
    result |= Rbyte;
    result <<= 6;

    int Gint = (int) Gtransform;
    uint8_t Gbyte = (uint8_t) Gint;
    result |= Gbyte;
    result <<= 5;

    int Bint = (int) Btransform;
    uint8_t Bbyte = (uint8_t) Bint;
    result |= Bbyte;

    return result;
}

Vimg make_vimg(char* fileName) {
    Vimg vimg = { 0 };
    
    if (!fileName) {
        printf("File name is NULL\n");
        return vimg;
    }

    //convert image into stream
    int width, height, channels;
    uint8_t* imgStream = stbi_load(fileName, &width, &height, &channels, 3); //force into RGB mode, no alpha for now
    if (!imgStream) {
        printf("Unable to load file %s, sorry :(\n", fileName);
        return vimg;
    }

    printf("Image %s loaded, with dimensions %ix%i\n", fileName, width, height);
    //check if the image is too big, remember that we must fit height and width into 2 uint8_ts each.

    if (width > ((1 << 16) - 1) || height > ((1 << 16) - 1)) {
        printf("Sorry, this image is too big, it must fit under or equal to %ux%u pixels", (1 << 16) - 1, (1 << 16) - 1);
        stbi_image_free(imgStream);
        return vimg;
    }

    //good to go!

    //make the file
    //get the file name
    char* outputName = (char*) calloc(strlen(fileName) + 10, sizeof(char));
    for (int i = 0; fileName[i] != '.' && i < strlen(fileName); i++) {
        outputName[i] = fileName[i];
    }
    strcat(outputName, ".vimg");
    
    const uint8_t compressionType = 0;
    
    // Make a character buffer
    // Use a Vector struct here to make life easier, copy from cottage
    dataVector v = dataVectorInit(1024);    
    dataVectorPushString(&v, "VIMG"); 
    
    dataVectorPush(&v, compressionType); 
    
    dataVectorPush(&v, (uint8_t)((uint16_t)(width)  >> 8)); 
    dataVectorPush(&v, (uint8_t)((uint16_t)(width)  & 0x00ff)); 
    dataVectorPush(&v, (uint8_t)((uint16_t)(height) >> 8)); 
    dataVectorPush(&v, (uint8_t)((uint16_t)(height) & 0x00ff)); 
    
    dataVectorPushString(&v, "VIMG"); 
    
    unsigned long imgSize = width * height * 3;
    for (unsigned long i = 0; i < imgSize;) {
        uint8_t
        R = imgStream[i++],
        G = imgStream[i++],
        B = imgStream[i++];

        uint16_t colour = convert(R, G, B);
        dataVectorPush(&v, (uint8_t)(colour >> 8));
        dataVectorPush(&v, (uint8_t)(colour & 0x00ff));
    }

    //finished
    printf("Generated image %s. Thank you, and goodbye.\n", outputName);
    stbi_image_free(imgStream);
    
    // Wrap it in a Vimg container and return it
    vimg.data = v.data;
    vimg.data_size = v.index;
    vimg.name = outputName;
    vimg.valid = true;
    return vimg;
}

#endif
