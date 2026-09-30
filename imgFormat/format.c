/*
This program will be used to convert any image into a custom format for the vjr.
It uses the stb_image.h library to convert an image into a raw RGB stream.
This stream is then converted into RGB565, and then placed intoa  file with a custom header file.

This filetype will be called... a V Image, shortened to .vimg, because im very creative, and obviously this is subject to change.

The header looks like so (in uint8_ts):
    
    String "VIMG" (4 uint8_ts), for identification and guard
    Compression type (1 uint8_t), a type of 0 is uncompressed.
    Full image width, in pixels (2 uint8_ts)
    Full image height, in pixels (2 uint8_ts)
    String "VIMG" (4 uint8_ts), guard


This totals 13 uint8_ts, subject to change in the future if needed.
Because its RGB565, each pixel will consist of 2 uint8_ts. for an uncompressed file, every 2-uint8_t is a pixel value.
*/

#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>

uint16_t convert(uint8_t R, uint8_t G, uint8_t B) {
    //ratios between max screen RGB values and standard RGB values
    const float Rk = 0.122;
    const float Gk = 0.248;
    const float Bk = 0.122;
    
    //apply the ratios
    float Rtransform = R * Rk;
    float Gtransform = G * Gk;
    float Btransform = B * Bk;

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



int main(int argc, char** argv) {
    if (argc <= 1) {
        printf("Please attach the name of the file you wish to convert. Thank you.\n");
        return 1;
    }

    char* fileName = argv[1];

    //convert image into stream
    int width, height, channels;
    uint8_t* imgStream = stbi_load(fileName, &width, &height, &channels, 3); //force into RGB mode, no alpha for now
    if (!imgStream) {
        printf("Unable to load file %s, sorry :(\n", fileName);
        return 1;
    }

    printf("Image %s loaded, with dimensions %ix%i\n", fileName, width, height);
    //check if the image is too big, remember that we must fit height and width into 2 uint8_ts each.

    if (width > ((1 << 16) - 1) || height > ((1 << 16) - 1)) {
        printf("Sorry, this image is too big, it must fit under or equal to %ux%u pixels", (1 << 16) - 1, (1 << 16) - 1);
        stbi_image_free(imgStream);
        return 1;
    }

    //good to go!

    //make the file
    //get the file name
    char* outputName = (char*) calloc(strlen(fileName) + 10, sizeof(char));
    for (int i = 0; fileName[i] != '.' && i < strlen(fileName); i++) {
        outputName[i] = fileName[i];
    }
    strcat(outputName, ".vimg");
    
    FILE* output = fopen(outputName, "wb");
    
    //writing the header
    
    const uint8_t compressionType = 0;
    fprintf(output, "VIMG");
    
    fputc(compressionType, output);
    
    fputc((uint8_t)((uint16_t)(width) >> 8), output);
    fputc((uint8_t)((uint16_t)(width) & 0x00ff), output);
    
    fputc((uint8_t)((uint16_t)(height) >> 8), output);
    fputc((uint8_t)((uint16_t)(height) & 0x00ff), output);
     
    fprintf(output, "VIMG");
    
    //actual conversion
    unsigned long imgSize = width * height * 3;
    for (unsigned long i = 0; i < imgSize;) {
        uint8_t
        R = imgStream[i++],
        G = imgStream[i++],
        B = imgStream[i++];

        uint16_t colour = convert(R, G, B);
//      printf("(%u,%u,%u) => 0x%x\n", R,G,B, colour);
        fputc((uint8_t)(colour >> 8), output);
        fputc((uint8_t)(colour & 0x00ff), output);
    }

    //finished
    printf("Generated image %s. Thank you, and goodbye.\n", outputName);
    stbi_image_free(imgStream);
    fclose(output);
    free(outputName);

    return 0;
}

