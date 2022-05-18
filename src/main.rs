extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use graphics::*;

// All constants must have their type explicitly declared
const CELL_ROWS: u32 = 12;
const CELL_COLS: u32 = 16;
const CELL_SIZE: u32 = 32;

// [f32; 4] represnts the type for a 4-length array of f32s, equivalent to [f32, f32, f32, f32].
// Similarly, the value here [1.0; 4] is a shorthand for [1.0, 1.0, 1.0, 1.0].
const WHITE: [f32; 4] = [1.0; 4];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const GREEN: [f32; 4] = [0.0, 0.8, 0.0, 1.0];

// Enums do not assign indices to values automatically (i.e. Horizontal = 0, Vertical = 1). Any calculations
// using these values must involve an explicit conversion function to be used beforehand.
enum Line {
    Horizontal,
    Vertical
}

enum Corner {
    TopLeft,
    TopRight,
    LowerLeft,
    LowerRight
}

// Defining a structure to hold the program state
pub struct App {
    drawing: DrawingPrimitives,
    // This is a 2-dimensional array, which will have dimensions of [CELL_ROWS x CELL_COLS]
    // Rust has no implicit data type casting (i.e. "promotion"), so here "as" allows for explicit data type casting.
    // Rust appears to want dimensions to have the "usize" data type.
    map: [[bool; CELL_COLS as usize]; CELL_ROWS as usize],
    last_mouse_pos: [f64; 2],
}

// Sub-struct to go inside the "App" struct. It's okay that this is declared here but used earlier.
// Rust doesn't mind about declaration order.
pub struct DrawingPrimitives {
    gl: GlGraphics, // OpenGL drawing backend.
    square: [f64; 4],
    inner_square: [f64; 4]
}

// "impl" block adds associated static/bound functions to a struct, essentially fleshing it out into a pseudo-class.
impl App {

    // Rust doesn't have true constructors (or true classes), and there is no language-based support for constructors.
    // Instead it is convention to simulate one via a regular method named "new" that returns a configured struct.
    pub fn new(opengl: glutin_window::OpenGL) -> App {
        let mut new_app = App {
            drawing: DrawingPrimitives {
                gl: GlGraphics::new(opengl),
                square: rectangle::square(0.0, 0.0, CELL_SIZE as f64),
                inner_square: rectangle::square(0.0, 0.0, CELL_SIZE as f64 * 0.9)
            },
            map: [[false; CELL_COLS as usize]; CELL_ROWS as usize],
            last_mouse_pos: [0.0; 2]
        };

        // Let's initialise with a Game of Life glider
        new_app.map[5][6] = true;
        new_app.map[5][7] = true;
        new_app.map[5][8] = true;
        new_app.map[4][8] = true;
        new_app.map[3][7] = true;

        return new_app;
    }

    // No "self" parameter = class function.
    // All function parameters require a type, and the return type is stated after the "->" in the function
    // signature, in this case a Matrix2d object from the math library.
    fn idx_transform(y_idx: f64, x_idx: f64,  c: graphics::Context) -> math::Matrix2d {
        // Need to explicitly convert CELL_SIZE from u32 to f64 before multiplying with another f64
        c.transform.trans(x_idx * CELL_SIZE as f64, y_idx * CELL_SIZE as f64)
    }

    // Also of note, no need for a "return" keyword if the last line doesn't have a semicolon, in which case it will be evaluated
    // as an expression and the result returned from the function (similar to the common syntax for lambdas)
    fn idx_offset_transform(y_idx: f64, x_idx: f64, c: graphics::Context) -> math::Matrix2d {
        c.transform.trans((x_idx - 0.5) * CELL_SIZE as f64, (y_idx - 0.5) * CELL_SIZE as f64) // This value will be returned from the function
    }

    // "Self" parameter = instance method. Self reference doesn't always have to be mutable, but in this case it does as we need
    // a mutable reference to the openGl object for passing to our drawing functions.
    fn draw_corner(&mut self, corner: Corner, y_idx: usize, x_idx: usize, colour: [f32; 4], c: graphics::Context) -> () {
        let to_loc = App::idx_offset_transform(y_idx as f64, x_idx as f64, c);
        match corner {
            Corner::TopLeft => circle_arc(colour, 1.0, 0.0, 1.57, self.drawing.square, to_loc.trans(CELL_SIZE as f64 * -0.5, CELL_SIZE as f64 * -0.5), &mut self.drawing.gl),
            Corner::TopRight => circle_arc(colour, 1.0,  1.57, 3.14, self.drawing.square, to_loc.trans(CELL_SIZE as f64 * 0.5, CELL_SIZE as f64 * -0.5), &mut self.drawing.gl),
            Corner::LowerLeft => circle_arc(colour, 1.0,  4.71, 6.28, self.drawing.square, to_loc.trans(CELL_SIZE as f64 * -0.5, CELL_SIZE as f64 * 0.5), &mut self.drawing.gl),
            Corner::LowerRight => circle_arc(colour, 1.0, 3.14, 4.71, self.drawing.square, to_loc.trans(CELL_SIZE as f64 * 0.5, CELL_SIZE as f64 * 0.5), &mut self.drawing.gl)
        }
    }

    // Here I've left off the return type, which will then default to "-> ()" (return void).
    fn draw_line(&mut self, line_dir: Line, y_idx: usize, x_idx: usize, colour: [f32; 4], c: graphics::Context) {
        let to_loc = App::idx_offset_transform(y_idx as f64, x_idx as f64, c);
        match line_dir {
            Line::Horizontal => line(colour, 1.0, [0.0, CELL_SIZE as f64 * 0.5, CELL_SIZE as f64, CELL_SIZE as f64 * 0.5], to_loc, &mut self.drawing.gl),
            Line::Vertical => line(colour, 1.0, [CELL_SIZE as f64 * 0.5, 0.0, CELL_SIZE as f64 * 0.5, CELL_SIZE as f64], to_loc, &mut self.drawing.gl)
        }
    }

    fn draw_cursor(&mut self, y_idx: usize, x_idx: usize, c: graphics::Context) -> () {
        let to_loc = App::idx_transform(y_idx as f64, x_idx as f64, c);
        rectangle(GREEN, self.drawing.square, to_loc, &mut self.drawing.gl);
        // Inner square is translated further by 5% of cell size
        rectangle(WHITE, self.drawing.inner_square, to_loc.trans(0.05 * CELL_SIZE as f64, 0.05 * CELL_SIZE as f64), &mut self.drawing.gl);
    }

    fn render(&mut self, args: &RenderArgs) -> () {
        // Get reference to openGl drawing object, and get drawing context for current frame
        let gl = &mut self.drawing.gl;
        let c = gl.draw_begin(args.viewport());

        // Clear the screen.
        clear(WHITE, gl);

        // Determine grid index of last mouse position and draw cursor
        let mouse_x_idx = (self.last_mouse_pos[0] as u32 / CELL_SIZE) as usize;
        let mouse_y_idx = (self.last_mouse_pos[1] as u32 / CELL_SIZE) as usize;
        self.draw_cursor(mouse_y_idx, mouse_x_idx, c);

        // Iterate over entire grid. Rust has an easy ".." syntax for ranges.
        for i in 0..((CELL_ROWS+1) as usize) {
            for j in 0..((CELL_COLS+1) as usize) {

                // Calculate pattern bit string based on values of available neighbours
                // Will only range from 0-15 so 8 bits is more than enough but is the smallest
                // integer type available.
                let mut value: u8 = 0;
                // Not on top edge
                if i > 0 {
                    // Not on left edge and top-left neighbour is on
                    if j > 0 && self.map[i-1][j-1] {
                        value += 8;
                    }
                    // Not on right edge and top-right neighbour is on
                    if j < CELL_COLS as usize && self.map[i-1][j] {
                        value += 4;
                    }
                }
                // Not on bottom edge
                if i < CELL_ROWS as usize{
                    // Not on left edge and bottom-left neighbour is on
                    if j > 0  && self.map[i][j-1] {
                        value += 2;
                    }
                    // Not on right edge and bottom-right neighbour is on
                    if j < CELL_COLS as usize && self.map[i][j] {
                        value += 1;
                    }
                }

                // Transitional tile to draw is decided by pattern bit string
                match value {
                    // Can match against multiple cases (in this case, two at a time)
                    1 | 14 => self.draw_corner(Corner::LowerRight, i, j, BLACK, c),
                    2 | 13 => self.draw_corner(Corner::LowerLeft, i, j, BLACK, c),
                    4 | 11 => self.draw_corner(Corner::TopRight, i, j, BLACK, c),
                    8 | 7 => self.draw_corner(Corner::TopLeft, i, j, BLACK, c),
                    10 | 5 => self.draw_line(Line::Vertical, i, j, BLACK, c),
                    12 | 3 => self.draw_line(Line::Horizontal, i, j, BLACK, c),
                    // Braces for a longer block for this match case
                    6 => {
                        self.draw_corner(Corner::LowerRight, i, j, BLACK, c);
                        self.draw_corner(Corner::TopLeft, i, j, BLACK, c);
                    },
                    9 => {
                        self.draw_corner(Corner::LowerLeft, i, j, BLACK, c);
                        self.draw_corner(Corner::TopRight, i, j, BLACK, c);
                    },
                    // Draw nothing for pattern bit string of 0 or 15
                    // Also catches all other values in the u8 value space, important as
                    // match will complain if it isn't exhaustive.
                    _ => ()
                }
            }
        }

        // Finish drawing
        self.drawing.gl.draw_end();
    }

    fn handle_mouse_move(&mut self, new_pos: [f64; 2]) {
        self.last_mouse_pos = new_pos;
    }

    fn handle_mouse_click(&mut self) {

        // Toy functional programming, creating a function that makes an iterator from an array
        // and then apply a map to it.
        let mut coords_to_indices = self.last_mouse_pos
            .iter()
            // Lambda functions take the form "|<inputs>| {output}"
            .map(|&coord| {coord as usize / CELL_SIZE as usize});

        // Multiple assignment. Haven't found a good way to unroll an iterator into a tuple, though.
        let (x_idx, y_idx) = (coords_to_indices.next().unwrap(), coords_to_indices.next().unwrap());

        // Toggle the clicked cell
        self.map[y_idx][x_idx] = !self.map[y_idx][x_idx];
    }
}

// Rust programs use the "main" function as the entry point 
fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "Marching Squares",
            [CELL_COLS as u32 * CELL_SIZE, CELL_ROWS as u32 * CELL_SIZE]
        )
        .graphics_api(opengl)
        .exit_on_esc(true)
        .controllers(true)
        .resizable(false) // Take that, James!
        .build()
        .unwrap();

    // Create a new game state object by calling the "new" simulated constructor
    let mut app = App::new(opengl);

    // lazy(true) triggers updates only on input. Without this, we would receive regular update events at a settable FPS.
    let mut events = Events::new(EventSettings::new().lazy(true));

    // Handle events as they continue to be provided by the window library
    // These follow the Rust "if let" pattern for one-off (non-exhaustive) matching
    while let Some(e) = events.next(&mut window) {
        // If the window wishes to render
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        // If the mouse moves
        if let Some(m) = e.mouse_cursor_args() {
            app.handle_mouse_move(m);
        }

        // If the left mouse button is clicked
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            app.handle_mouse_click();
        }
    }
}