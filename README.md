![repository-card](repository-card.png)

# rustymap
Render Minecraft Maps

## This is a learning project

This project is primarily a learning tool for me to learn [Rust](https://doc.rust-lang.org/book/). I mostly work with other languages, such as PHP, Python, and JavaScript, all of which were self-taught. I learn through immersion, and this project is an example of this. The following are my goals and how I intend to meet them:

### Learning goals

#### Proper Syntax
You can't have a working codebase if the syntax is wrong!

#### Variables, Types, and Data Structures
Minecraft NBT data provides ample opportunity to learn how to convert data between different types, parse data from raw bytes, and much more.

#### File Input/Output
This will be part of the process of reading data from the Minecraft files in raw format to parse as NBT data, as well as manipulating and saving image data back onto the filesystem.

## Roadmap / ToDo List
This is a list of things I have done/need to do which is updated as I think of things

### Configuration Parser
- [x] Decide on config structure
- [x] Create full valid config file
- [x] Implement default values for missing settings
- [ ] Validate defined render dimension
- [x] Add checks and exits for invalid inputs
- [ ] Verify/create given output directory

### Region Files
- [x] Go through header table to identify chunks
- [x] Decompress raw bytes to NBT bytes
- [x] Implement NBT parser for region chunks

### Chunks & Sections
- [x] Extract data version of chunk
- [x] Collect x, y, z coordinates
- [x] Compile block states pallet list
- [x] Calculate block and sky lighting values

### Named Binary Tags (NBT)
- [x] Identify each type of NBT format tag
- [x] Create ability to ingest each data type
- [x] Create an NBT struct of tags _(is this necessary?)_

### Textures
- [x] Check for valid default/given texture path
- [x] Add directory traversal methods for `.jar`/`.zip` archives
- [x] Create cache directory named by hash of texture container file
- ~~[ ] Add drill-down methods for extracting block textures on-demand~~
- [x] Extract all textures into cache directory
- [x] Create staging area for texture files _(aka cache)_

### Block States & Models

_I need to figure out how to extract the blockstate and model data from the minecraft jar so as to dynamically apply the
block textures to the render and not have to manually hard-code these values. This is the best long-term solution for
project maintenance, but means extra work up-front._

- [x] Add `jar\assets\minecraft\blockstates\*` content to cache
- [x] Add `jar\assets\minecraft\models\block\*` content to cache
- ~~Figure out how blockstates correlate to nbt data~~
- [ ] Determine how to map textures from model data
 
#### Sample blockstate data: `grass_block.json`

```json
{ "variants": {
    "snowy=false": [
      {
        "model": "minecraft:block/grass_block"
      },
      {
        "model": "minecraft:block/grass_block",
        "y": 90
      },
      {
        "model": "minecraft:block/grass_block",
        "y": 180
      },
      {
        "model": "minecraft:block/grass_block",
        "y": 270
      }
    ],
    "snowy=true": {
      "model": "minecraft:block/grass_block_snow"
    }
  }
}
```

#### Sample model data: `grass_block.json`

```json
{   "parent": "block/block",
    "textures": {
        "particle": "block/dirt",
        "bottom": "block/dirt",
        "top": "block/grass_block_top",
        "side": "block/grass_block_side",
        "overlay": "block/grass_block_side_overlay"
    },
    "elements": [
        {   "from": [ 0, 0, 0 ],
            "to": [ 16, 16, 16 ],
            "faces": {
                "down":  { "uv": [ 0, 0, 16, 16 ], "texture": "#bottom", "cullface": "down" },
                "up":    { "uv": [ 0, 0, 16, 16 ], "texture": "#top",    "cullface": "up", "tintindex": 0 },
                "north": { "uv": [ 0, 0, 16, 16 ], "texture": "#side",   "cullface": "north" },
                "south": { "uv": [ 0, 0, 16, 16 ], "texture": "#side",   "cullface": "south" },
                "west":  { "uv": [ 0, 0, 16, 16 ], "texture": "#side",   "cullface": "west" },
                "east":  { "uv": [ 0, 0, 16, 16 ], "texture": "#side",   "cullface": "east" }
            }
        },
        {   "from": [ 0, 0, 0 ],
            "to": [ 16, 16, 16 ],
            "faces": {
                "north": { "uv": [ 0, 0, 16, 16 ], "texture": "#overlay", "tintindex": 0, "cullface": "north" },
                "south": { "uv": [ 0, 0, 16, 16 ], "texture": "#overlay", "tintindex": 0, "cullface": "south" },
                "west":  { "uv": [ 0, 0, 16, 16 ], "texture": "#overlay", "tintindex": 0, "cullface": "west" },
                "east":  { "uv": [ 0, 0, 16, 16 ], "texture": "#overlay", "tintindex": 0, "cullface": "east" }
            }
        }
    ]
}

```

_Further thoughts: the block model json files appear to take the block name and link the faces to associated textures.
The problem now appears to be that there isn't a completely consistent block:texture mapping structure, as some model
data will link to other model data, which then substitutes some texture location mappings with other strings, example:_

#### Sample model data: `deepslate.json`

```json
{
  "parent": "minecraft:block/cube_column",
  "textures": {
    "end": "minecraft:block/deepslate_top",
    "side": "minecraft:block/deepslate"
  }
}
```

#### Sample "parent" model data: `cube_column.json`
```json
{
    "parent": "block/cube",
    "textures": {
        "particle": "#side",
        "down": "#end",
        "up": "#end",
        "north": "#side",
        "east": "#side",
        "south": "#side",
        "west": "#side"
    }
}
```

#### Sample "grandparent" model data: `cube.json`

```json
{
    "parent": "block/block",
    "elements": [
        {   "from": [ 0, 0, 0 ],
            "to": [ 16, 16, 16 ],
            "faces": {
                "down":  { "texture": "#down", "cullface": "down" },
                "up":    { "texture": "#up", "cullface": "up" },
                "north": { "texture": "#north", "cullface": "north" },
                "south": { "texture": "#south", "cullface": "south" },
                "west":  { "texture": "#west", "cullface": "west" },
                "east":  { "texture": "#east", "cullface": "east" }
            }
        }
    ]
}
```

### Rendering
- [X] Render overworld chunk y-slice top-down
- [ ] Render full overworld chunk top-down without fluid
- [ ] Render full overworld chunk top-down with fluid
- [ ] Render nether top-down sans roof
- [ ] Render any isometric chunk section
- [ ] Render any isometric full chunk
- [ ] Figure out how to handle non-solid blocks _(fence, stairs, etc.)_
- [ ] Figure out how to ignore hidden blocks
- [ ] Add settings to adjust rotation

### Browser Viewer
- [ ] ????? _(obviously JavaScript is the defacto standard for these types of projects, but Rust can do [web assembly](https://www.rust-lang.org/what/wasm) so I guess I could dive into that?)_
