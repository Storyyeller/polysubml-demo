// ASCII Mandelbrot Set in PolySubML
// Generates a simple ASCII art representation of the Mandelbrot set

// Helper function to compute z^2 + c for complex numbers
let complex_step = fun {zr; zi; cr; ci} -> {
  zr = zr *. zr -. zi *. zi +. cr;
  zi = 2.0 *. zr *. zi +. ci
};

// Function to test if a point diverges (escapes)
let rec mandel_iter = fun {zr; zi; cr; ci; max_iter; iter} ->
  if iter >= max_iter then
    iter
  else if zr *. zr +. zi *. zi >. 4.0 then
    iter
  else (
    let z_new = complex_step {zr; zi; cr; ci};
    mandel_iter {
      zr = z_new.zr;
      zi = z_new.zi;
      cr; ci; max_iter;
      iter = iter + 1
    }
  );

// Function to get ASCII character based on iteration count
let get_char = fun iter ->
  if iter >= 20 then " "
  else if iter >= 15 then "."
  else if iter >= 10 then ":"
  else if iter >= 5 then "*"
  else "#";

// Print a single row
let print_row = fun {y; width; height; x_min; x_max; y_min; y_max} -> (
  let vars = {mut x = 0.0; mut line = ""};
  loop if vars.x >=. 60.0 then `Break 0 else (
    let x_float = vars.x;
    let cr = x_min +. (x_max -. x_min) *. x_float /. (width -. 1.0);
    let ci = y_min +. (y_max -. y_min) *. y /. (height -. 1.0);
    let iter = mandel_iter {zr=0.0; zi=0.0; cr; ci; max_iter=20; iter=0};
    let char = get_char iter;
    vars.line <- vars.line ^ char;
    vars.x <- vars.x +. 1.0;
    `Continue 0
  );
  print vars.line;
  0
);

// Main function to generate the Mandelbrot set
let mandelbrot = fun _ -> (
  let width = 60.0;
  let height = 30.0;
  let x_min = -2.5;
  let x_max = 1.0;
  let y_min = -1.0;
  let y_max = 1.0;

  let vars = {mut y = 0.0};
  loop if vars.y >=. 30.0 then `Break 0 else (
    let y_float = vars.y;
    print_row {
      y = y_float;
      width; height;
      x_min; x_max; y_min; y_max
    };
    vars.y <- vars.y +. 1.0;
    `Continue 0
  );
  0
);

// Run the program
mandelbrot {};