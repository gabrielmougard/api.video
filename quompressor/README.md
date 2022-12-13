

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

## TODO

* Write .QIM format spec
* Write a couple of examples (if using CLI) 
* Write python FFIs (with PyO3)
* Write Gradio app with python FFIs to have a nice user-friendly app for the demo
* At this point, it is ready to be shown and we need to write the actual tech talk.