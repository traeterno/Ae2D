# Ae2d (Aeterno's 2D)
### Ae2d is a game engine which main target is to run on low-end PCs without any problems while giving decent gaming and visual experience.

## Development documentation

- [Building the engine](#building)
- [Initialization](#initialization)
- [Nice texts](#nice-texts)
- [Writing UI](#writing-ui)
- [Writing animators](#writing-animators)

## Building
Building is successful on Linux and Windows. On both platforrms you need Rust toolchain and SDL2 library with extra SDL2_image and SDL2_ttf package. Simply run `cargo build --release` and Cargo will do everything for you.

## Initialization
For successful engine startup you need:
- [Configuration file](#configuration-file);
- [Color palette file](#color-palette);
- [Main UI file](#main-ui).


### Configuration file
The main config file is located at `res/global/config.json`. It contains following sections:
- `init` - Variables needed to create window and start the engine;
- `optional` - Variables that are not necessary for the engine, but can be used by it;
- `custom` - Global variables which can be accessed from anywhere in the engine.

### Example:
```json
{
	"init": {
		"title": "ae2d",
		"style": "fullscreen",
		"size": { "w": 1920, "h": 1080 }
	},
	"optional": {
		"position": { "x": 860, "y": 540 }
	},
	"custom": {
	    "parameter": "value"
	}
}
```

#### `init` Entries:
- `title` - Initial name of the window, can be any string value;
- `style` - Style of the window:
	- `default` - Simple window, can't be resized;
	- `resizable` - Simple window, can be resized;
	- `borderless` - Window without decorations;
	- `fullscreen` - fullscreen window.
- `size` - Size of the window. Should contain width and height of the screen in pixels, i.e. `{ "w": 1920, "h": 1080}`. It won't be used with the `fullscreen` style.
#### `optional` Entries:
- `position` - Position of the window. Should contain X and Y coordinates, i.e `{ "x"; 860, "y": 540 }`. If not provided, the window will appear in the center of the screen;
- `hideCursor [true/false]` - Show/hide cursor;
- `lockCursor [true/false]` - Lock/unlock the cursor in center of the window;
- `vsync [true/false]` - Enable/disable V-Sync;
- `minDeltaTime` - Minimal elapsed time between frames: 0.033 seconds at 30 FPS, 0.017 seconds at 60 FPS etc;
- `maxDeltaTime` - Maximal elapsed time between frames.
#### `custom` Entries are parsed automatically: If the value can be interpreted as integer or floating-point number, then it will be so, otherwise it is contained as string.

### Color palette
ae2d lets you [color](#text-coloring) your texts, but for that you need a palette. It can be defined in `res/global/colors.json` file as follows:

```json
{
	"colorName": {
		"r": 255,
		"g": 255,
		"b": 255,
		"a": 255
	}
}
```

After that you can use these colors via their names.

### Main UI

The first thing engine does after reading configuration and getting colors is loading the main menu. It is located at `res/ui/mainMenu.xml`. You can read more about it [here](#writing-ui).

## Nice texts

### Text formatting

ae2d allows you to style your texts via markdown:
- `^()` - Regular text;
- `^(*)` - **Bold text**;
- `^(/)` - *Italic text*;
- `^(_)` - <ins>Underlined text</ins>;
- `^(-)` - ~~Strikethrough text~~.

Styles also can be combined, i.e. `^(* /)` - ***Both bold and italic***.

The text will contain the style that was written in front of it; In case you want to reset the style, you have to write `^()` before the part that has to be regular:
> This is ^(*)**a bold text, and** ^(/)*this is an italic text.*

> Are you ^(* _ /)***<ins>Sure</ins>*** ^()that you want to quit programming? ^(*)**Yes** ^(/)*No*

### Text coloring

When setting the style for text, you can also set the color for that text part: `^(clr=red)`. For more information about colors look [here](#color-palette). The default text color is white.

## Writing UI
UI is basically an XML file. Here is an example:
```xml
<object name="root" script="scripts/mainMenu.lua" order="itc">
	<image anim="anims/mainMenu.json" />
	<text font="fonts/main.fnt" size="24">Some text</text>
	<var name="visible" num="1" str="true" />
	
	<object name="button" script="scripts/btn.lua" order="itc">
		...
	</object>
</object>
```

And now more depth:
- `object` - Container for image, text and other objects. Should be the start of UI file. Its parameters are:
- - `name` - Name of the object, may be useful in scripts;
- - `script` - Path to the Lua script that will be executed by the object. May be not declared.
- - `order` - Drawing order of the object: `i` - image, `t` - text, `c` - inner objects. `itc` order will first draw image, then text and the child objects at the end. If you don't want the object to draw something, you can specify `_` instead of parameter i.e. `t__` to draw only the text part. **This parameter should always be 3 characters long.**
- `image` - Node for specifying image animation. Its only parameter is `anim` to specify the [animator file](#writing-animators).
- `text` - Node for specifying text parameters:
- - `font` - Path to the font file. The font should be represented as PNG+XML bitmap font. **TODO: Add the link to the converter**
- - `size` - The size of the font.
- `var` - Custom variable for the object which can be used in the script. Its parameters names speak for themselves.

## Writing animators
Animator is a simple JSON file containing essential information about texture and animations.
### Example:
```json
{
	"texture": "res/tex/test.png",
	"frame": { "x": 32, "y": 32 },
	"animations": [
		{
			"name": "main",
			"repeat": 0,
			"frames": [
				{ "frame": 0, "duration": 0.5 },
				{ "frame": 1, "duration": 0.5 }
			]
		}
	]
}
```

- `texture` - Path to the sprite-sheet;
- `frame` - The size of the frame;
- `animations` - The animations array:
- - `name` - The name of animation;
- - `repeat` - How many times the animation should repeat. If set to 0, the anim will play forever.
- - `frames` - The frames list:
- - - `frame` - The ID of frame in the sprite-sheet, counting from top-left right-downward, starting from 0 i.e. The first frame is 0, right one is 1, down one is 2, right-down is 3 etc.
- - - `duration` - For how long the frame should play in seconds.# Ae2D
