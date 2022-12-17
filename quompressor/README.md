

# Quompressor

<p align="center">
  <img src="./doc/api.video-quadtree.gif" />
  <p align="center">The principle of quadtree compression : each "square" is a single color.</p>
</p>

```bash
   ____                                                                      
  / __ \ __  __ ____   ____ ___   ____   _____ ___   _____ _____ ____   _____
 / / / // / / // __ \ / __ `__ \ / __ \ / ___// _ \ / ___// ___// __ \ / ___/
/ /_/ // /_/ // /_/ // / / / / // /_/ // /   /  __/(__  )(__  )/ /_/ // /    
\___\_\\__,_/ \____//_/ /_/ /_// .___//_/    \___//____//____/ \____//_/     
                              /_/                                            
```
This tool aims at compressing PNG pictures in a funny way. I'm not sure that this is extremely performant
but it gives a rough idea about how image compression and binary representation can work in Rust. This
project is also educational and can be used as a support for a Rust tech talk (cf. the amazing Gradio amazing UI for the demos (still WIP) !) 

## QIM binary format spec

You can find the spec for the QIM format (intermediate representation of the compressed data on disk) [here](./doc/QIM_spec.md)

# Example

```bash
# generate the .QIM intermediate binary representation
./target/release/quompressor -i examples/kitchen-2048x2048.png examples/kitchen-2048x2048_loss.qim 

# generate the compressed .PNG
./target/release/quompressor -f examples/kitchen-2048x2048_loss.qim -w 2048

du -h kitchen-2048x2048.png # 5.1M
du -h kitchen-2048x2048_loss.png # 4.1M (~ 20% smaller with same size and still with a very decent quality)
```

## Build instructions

* If you wish to build the CLI binary, just do `cargo build --release`. The output binary is at `target/release/quompressor`

* If you wish to build the python app with the shared Rust lib :
  * Create a virtual environment : `python3 -m venv .env && source .env/bin/activate`
  * Install `pip-tools` : `pip install pip-tools`
  * You can use the compiled `requirements/base.txt` to install the project python deps with `pip install -r requirements/base.txt` or regenerate the list of python dependencies with `pip-compile requirements/base.in` and then install the recompiled dependencies with `pip install -r requirements/base.txt`
  * Compile the shared lib and generate FFIs for CPython using : `maturin develop`
  * Now, try to see if the `quompressor` Python module is available
  
  ```python
  from quompressor import compress # If this is OK, the lib has been ported to a CPython module
  ```


  * You can execute the python example code : `python main.py`. It should give you this :

  ```bash
  The input file ./examples/kitchen-2048x2048.png size is 5339604 bytes
  The output file ./examples/kitchen-2048x2048-python.png size is 4069091 bytes
  The compression ratio is : 0.23794142786618633%
  ```
