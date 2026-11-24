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

#include "../../shared/vimg.h"
#include <stdio.h>

int main(int argc, char** argv) {
    if (argc <= 1) {
        printf("Please attach the name of the file you wish to convert. Thank you.\n");
        return 1;
    }
    
    // Ask questions about font
    // Glyph height, Glyph width
    
    // char response[100] = {0};
    uint16_t height = 0, width = 0, glyphs_per_col = 0, glyphs_per_row = 0;

    printf("Glyph width: ");
    scanf("%u", &width);

    printf("Glyph height: ");
    scanf("%u", &height);
    
    printf("Glyphs per column: ");
    scanf("%u", &glyphs_per_col);
    
    printf("Glyph per row: ");
    scanf("%u", &glyphs_per_row);


    Args args = parse_args(argc, argv);
    
    if (!args.input_file || !args.output_file) {
        printf("Please provide an input name and an output name.\n");
        return 1;
    }

    Vimg vimg = make_vimg(args.input_file);
    
    if (vimg.valid && vimg.name && vimg.data) {
        // Write to file
        FILE* outputFile = fopen(args.output_file, "wb");
        
        // Font specific data
        fprintf(outputFile, "VFONT");
        fwrite(&width,          sizeof(uint16_t), 1, outputFile);
        fwrite(&height,         sizeof(uint16_t), 1, outputFile);
        fwrite(&glyphs_per_col, sizeof(uint16_t), 1, outputFile);
        fwrite(&glyphs_per_row, sizeof(uint16_t), 1, outputFile);
        fprintf(outputFile, "VFONT");
        
        // Image specific data
        fwrite(vimg.data, sizeof(uint8_t), vimg.data_size, outputFile);

        fclose(outputFile);
    }
    else {
        printf("Could not make VIMG\n");
        return 1;
    }

    return 0;
}

