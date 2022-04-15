# ROS2 Turtlesim with eclipse zehno.

inspired by [turtlesim with bevy by @quietlychris](https://github.com/quietlychris/turtlesim) and bevy [2d rotaion example](https://github.com/bevyengine/bevy/blob/main/examples/2d/rotation.rs)

launch turtlesim node
~~~sh
ros2 run turtlesim turtlesim_node
~~~

launch turtlesim telepo key
~~~sh
ros2 run turtlesim turtle_teleop_key
~~~

set up zenoh bridge dds
~~~sh
zenoh-bridge-dds  -l tcp/0.0.0.0:7447
~~~

try this project (may need modify connect endpoint if run in different network)
~~~sh
cargo run 
~~~

may also try zenoh ros2 [rust teleop example](https://github.com/wolfboyyang/zenoh-demos/tree/tokio/ROS2/zenoh-rust-teleop)
~~~sh
cargo run -- -e tcp/127.0.0.1:7447
~~~
