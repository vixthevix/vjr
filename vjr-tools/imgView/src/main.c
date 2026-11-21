/*
This program will be used to view images of my format, right now called V Images.
Uses raylib and rgb565->rgb888 conversion to display the image.
*/

#include <raylib.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>

Color convert(uint16_t colour) {
    //split up the RGB components back into unsigned chars, then divide by k values.
    
    //ratios between max screen RGB values and standard RGB values
    const float Rk = 0.122;
    const float Gk = 0.248;
    const float Bk = 0.122;
    
    uint8_t Bbyte = (uint8_t)(colour & 0b11111);
    colour >>= 5;
    
    uint8_t Gbyte = (uint8_t)(colour & 0b111111);
    colour >>= 6;
    
    uint8_t Rbyte = (uint8_t)(colour & 0b11111);
    colour >>= 5;

    uint16_t Rint = (uint16_t)Rbyte;
    uint16_t Gint = (uint16_t)Gbyte;
    uint16_t Bint = (uint16_t)Bbyte;

    float RTransform = ((float)Rint) / Rk;
    float GTransform = ((float)Gint) / Gk;
    float BTransform = ((float)Bint) / Bk;

    Color result = {
        .r = (uint8_t)RTransform,
        .g = (uint8_t)GTransform,
        .b = (uint8_t)BTransform,
        .a = 255
    };

    return result;
}


typedef struct VImg {
    uint16_t width;
    uint16_t height;
    uint8_t compType;
    FILE* file;
} VImg;

VImg* readVImg(const char* fileName) {
    uint16_t width = 0, height = 0;
    uint8_t compType = 0;

    FILE* file = fopen(fileName, "rb");
    if (!file) return NULL;

    //read 13 byte header
    uint8_t header[13];
    fread(header, 13, sizeof(uint8_t), file);
    
    //check the guards
    char 
    fguard[5] = {0},
    bguard[5] = {0};
    for (int i = 0, j = 9; i < 4; i++, j++) {
        fguard[i] = header[i];
        bguard[i] = header[j];
    }

    if (strcmp(fguard, "VIMG") != 0 || strcmp(bguard, "VIMG")) {
        //guards are incorrect.
        fclose(file);
        return NULL;
    }

    //guards are good, so we default to reading the values
    compType = header[4];
    width = ((uint16_t)(header[5]) << 8) | (header[6]);
    height = ((uint16_t)(header[7]) << 8) | (header[8]);    

    VImg* image = (VImg*) malloc(sizeof(VImg));
    image->width = width;
    image->height = height;
    image->compType = compType;
    image->file = file;

    return image;

}


int main(int argc, char** argv) {
    if (argc <= 1) {
        printf("Please attach the name of the file you wish to view. Thank you.\n");
        return 1;
    }

    char* fileName = argv[1];

    //ensure its of type vimg
    VImg* image = readVImg(fileName);
    if (!image) {
        printf("This is not a valid VImg. Thank you.\n");
        return 1;
    }

    //with this valid VImg, we can start up raylib to begin drawing.
    //our window dimensions will match the images
    InitWindow(image->width, image->height, fileName);
    SetTargetFPS(60);

    //draw the image
    //we will read chunks of image->width bytes and display them.
    printf("Image displayed. Thank you.\n");

    //loop to keep window open
    bool drawn = false;
    while (!WindowShouldClose()) {
	    BeginDrawing();
	    
		if (!drawn) { // Draw once
			drawn = true;
			ClearBackground(WHITE);
			for (unsigned long y = 0; y < image->height; y++) {
				uint8_t buffer[image->width * 2];
				fread(buffer, image->width * 2, sizeof(uint8_t), image->file);
				for (unsigned long x = 0, i = 0; x < image->width; x++) {
					//every 2 bytes in buffer is a pixel. we must convert to rgb888, and then display it.
					uint8_t high = buffer[i++], low = buffer[i++];
					uint16_t colour = ((uint16_t)(high) << 8) | low;
					Color displayColour = convert(colour);
					DrawPixel(x, y, displayColour);
				}
			}  
	    }

	    EndDrawing();
    }

    CloseWindow();
    
    //free everything
    fclose(image->file);
    free(image);


    return 0;
}

