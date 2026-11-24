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


    uint8_t RTransform = (Rbyte) << 3;
    uint8_t GTransform = (Gbyte) << 2;
    uint8_t BTransform = (Bbyte) << 3;

    Color result = {
        .r = RTransform,
        .g = GTransform,
        .b = BTransform,
        .a = 255
    };
    
//    printf(
//        "Colour:\n"
//        "\tR = %u\n"
//        "\tG = %u\n"
//        "\tB = %u\n"
//    , result.r, result.g, result.b);

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
    
    // Create a buffer to store the image and pass it to Raylib
    
    // List of raylib pixels 
    Color* pixels = (Color*)malloc(image->width * image->height * sizeof(Color));
    uint8_t* row_buf = (uint8_t*)malloc(image->width * 2);

    for (unsigned long y = 0; y < image->height; y++) {
        fread(row_buf, 1, image->width * 2, image->file);
        for (unsigned long x = 0; x < image->width; x++) {
            uint16_t colour = ((uint16_t)row_buf[x * 2] << 8) | row_buf[x * 2 + 1];
            pixels[y * image->width + x] = convert(colour);
        }
    }
    free(row_buf);

    // Passing on to raylib
    Image rlImg = {
        .data = pixels,
        .width = image->width,
        .height = image->height,
        .mipmaps = 1,
        .format = PIXELFORMAT_UNCOMPRESSED_R8G8B8A8
    };
    Texture2D texture = LoadTextureFromImage(rlImg);
    free(pixels); 

    //draw the image
    //we will read chunks of image->width bytes and display them.
    printf("Image displayed. Thank you.\n");

    //loop to keep window open
    bool drawn = false;
    while (!WindowShouldClose()) {
	    BeginDrawing();
        ClearBackground(WHITE);
        DrawTexture(texture, 0, 0, WHITE);
	    EndDrawing();
    }
    
    UnloadTexture(texture);
    CloseWindow();
    
    //free everything
    fclose(image->file);
    free(image);

    return 0;
}

