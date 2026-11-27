/*
This program will be used to convert any image into a custom format for the vjr.
It uses the stb_image.h library to convert an image into a raw RGB stream.
This stream is then converted into RGB565, and then placed intoa  file with a custom header file.

This filetype will be called... a V Image, shortened to .vfont, because im very creative, and obviously this is subject to change.

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

typedef struct Vfont {
    char* data;
    uint32_t data_size;
    bool valid;
} Vfont;

// uint16_t height = 0, width = 0, glyphs_per_col = 0, glyphs_per_row = 0;

Vfont make_vfont(char* fileName, uint16_t glyph_width, uint16_t glyph_height, uint16_t glyphs_per_col, uint16_t glyphs_per_row) {
    Vfont vfont = { 0 };
    
    if (!fileName) {
        printf("File name is NULL\n");
        return vfont;
    }

    //convert image into stream
    int width, height, channels;
    uint8_t* imgStream = stbi_load(fileName, &width, &height, &channels, 4); // Force into RGBA mode
    if (!imgStream) {
        printf("Unable to load file %s, sorry :(\n", fileName);
        return vfont;
    }

    printf("Image %s loaded, with dimensions %ix%i\n", fileName, width, height);
    //check if the image is too big, remember that we must fit height and width into 2 uint8_ts each.

    // if (width > ((1 << 16) - 1) || height > ((1 << 16) - 1)) {
    //     printf("Sorry, this image is too big, it must fit under or equal to %ux%u pixels", (1 << 16) - 1, (1 << 16) - 1);
    //     stbi_image_free(imgStream);
    //     return vfont;
    // }
    
    /*
    VFONT STRUCTURE
        "VFONT" guard
        Image width     (2 bytes)
        Image height    (2 bytes)
        Glyph width     (2 bytes)
        Glyph height    (2 bytes)
        Glyphs per cols (2 bytes)
        Glyphs per rows (2 bytes)
        "VFONT" guard
    */

    dataVector v = dataVectorInit(1024);    
    dataVectorPushString(&v, "VFONT");

    dataVectorPush(&v, (uint8_t)(width >> 8));
    dataVectorPush(&v, (uint8_t)(width & 0x00ff));

    dataVectorPush(&v, (uint8_t)(height >> 8));
    dataVectorPush(&v, (uint8_t)(height & 0x00ff));

    dataVectorPush(&v, (uint8_t)(glyph_width >> 8));
    dataVectorPush(&v, (uint8_t)(glyph_width & 0x00ff));

    dataVectorPush(&v, (uint8_t)(glyph_height >> 8));
    dataVectorPush(&v, (uint8_t)(glyph_height & 0x00ff));

    dataVectorPush(&v, (uint8_t)(glyphs_per_col >> 8));
    dataVectorPush(&v, (uint8_t)(glyphs_per_col & 0x00ff));

    dataVectorPush(&v, (uint8_t)(glyphs_per_row >> 8));
    dataVectorPush(&v, (uint8_t)(glyphs_per_row & 0x00ff));

    dataVectorPushString(&v, "VFONT");
   

    // For fonts, we will use a monochrome font
    // 1 bit = 1 pixel, so 1 byte = 8 pixels

    unsigned long imgSize = width * height * 4;
    uint8_t cur_byte = 0, write_byte = 0;
    for (unsigned long i = 0; i < imgSize;) {
        uint8_t
        R = imgStream[i++],
        G = imgStream[i++],
        B = imgStream[i++],
        A = imgStream[i++];
        
		// printf("Pixil %lu: (%u, %u, %u, %u)\n", i / 4, R, G, B, A);

        write_byte <<= 1;
        if (R == 0 && G == 0 && B == 0) {
            write_byte |= 1;
        }
		else {
			write_byte |= 0;
		}
        
        cur_byte = (cur_byte + 1) % 8;
        if (cur_byte == 0) {
            // Reached 8 bits read, so write
            dataVectorPush(&v, write_byte);
            write_byte = 0;
        }
        
    }
    // Write remaining bits
    if (cur_byte != 0) {
        // Shift write byte up to make most significant
        write_byte <<= (8 - cur_byte);
        dataVectorPush(&v, write_byte);

    }
    
    // Wrap in Vfont container and return
    vfont.data = v.data;
    vfont.data_size = v.index;
    vfont.valid = true;
    stbi_image_free(imgStream);
    
    return vfont;
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

    Vfont vfont = make_vfont(args.input_file, width, height, glyphs_per_col, glyphs_per_row);
    
    if (vfont.valid && vfont.data) {
        // Write to file
        FILE* outputFile = fopen(args.output_file, "wb");
        fwrite(vfont.data, sizeof(uint8_t), vfont.data_size, outputFile);
        fclose(outputFile);
        printf("Generated image %s. Thank you, and goodbye.\n", args.output_file);
    }
    else {
        printf("Could not make VFONT\n");
        return 1;
    }

    return 0;
}

