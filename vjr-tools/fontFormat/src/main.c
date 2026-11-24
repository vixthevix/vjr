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

// Read a number up to a limit of 9999.
// Returns the number if valid with no extraneous characters other
// than leading zeroes, or 0xFFFFu otherwise.  An empty input is
// not considered valid.
uint16_t read_number(void)
{
  uint16_t value_input = 0;
  int ch = getchar();
  if (ch < 0 || ch == '\n')
    return 0xFFFFu;
  do
  {
    if (ch >= '0' && ch <= '9' && value_input <= 9999)
      value_input = value_input*10 + (ch - '0');
    else
      value_input = 0xFFFFu;
    ch = getchar();
  } while(ch >= 0 && ch != '\n');
  return value_input;
}

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
    width = read_number();

    printf("Glyph height: ");
    height = read_number();
    
    printf("Glyphs per column: ");
    glyphs_per_col = read_number();
    
    printf("Glyph per row: ");
    glyphs_per_row = read_number();
    
    printf("width: %u, height: %u", width, height);

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

        fputc((uint8_t)(width >> 8),     outputFile);
        fputc((uint8_t)(width & 0x00ff), outputFile);

        fputc((uint8_t)(height >> 8),     outputFile);
        fputc((uint8_t)(height & 0x00ff), outputFile);
        
        fputc((uint8_t)(glyphs_per_col >> 8),     outputFile);
        fputc((uint8_t)(glyphs_per_col & 0x00ff), outputFile);
        
        fputc((uint8_t)(glyphs_per_row >> 8),     outputFile);
        fputc((uint8_t)(glyphs_per_row & 0x00ff), outputFile);

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

