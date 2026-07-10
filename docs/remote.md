# Remote

Snout supports a remote control interface using OSC, allowing you to control various features of the application remotely.
A binary (`snout-remote`) is provided that can be used to control Snout. Although any application that can send OSC messages will be able to do the same.

## Configuration

Add the following to your config file, and set the listening address to the address that your controller program is sending OSC messages to.

```toml
[control]
listen = "127.0.0.1:9500"
```

The [snout-remote](remote.md) utility sends osc messages to `127.0.0.1:9500` by default.

## `snout-remote` usage

> [!NOTE]
> The `<shape>` arguments are case sensitive, a list of all shapes can be found [below](#available-face-shapes)

`snout-remote` sends osc messages to `127.0.0.1:9500` by default, but can be set to use a different address through the `-t` flag, like so:

```sh
snout-remote -t 127.0.0.1:9004 <...>
```

### Setting face bounds

To set the bounds of a face shape, run the following command:

```sh
snout-remote face-bounds <shape> <lower> <upper>
```

An example for setting the bounds of a shape can be found below.

```sh
snout-remote face-bounds "MouthLeft" 0.4 1.0
```

### Face auto calibration

> ![NOTE]
> Bounds set through the auto calibration process are *not* saved persistently.
> Adding the `-v` flag when launching `snout-cli` will let you see the set bounds and manually apply them permanently in the configuration file.

You can auto calibrate the lower bounds of all face shapes using the `face-calibrate` command.
This will take a 100 frames worth of data (about 3 seconds at 30fps) and use it to determine the lower bounds of the face shapes.

Make sure to keep a neutral face through the calibration cycle.

```sh
snout-remote face-calibrate-lower [--frames <n>]
```

### Face calibrate upper bound

You can calibrate the upper bound of a face shape using the `face-calibrate-upper` command.
Optionally you can override the amount of frames used for calibration, using the `--frames <n>` flag.

```sh
snout-remote face-calibrate-upper <shape> [--frames <n>]
```

This will capture N frames (default 100) and use them to determine the upper bound of the face shape.
Try and keep the maximum of the particular shape you're trying to calibrate.

## API

### Set face bounds

Set the bounds of a face shape.

```osc
/snout/face/bounds <shape> <lower> <upper>
```

### Start face auto calibration

Start the auto calibration process for the face.

```osc
/snout/face/calibrate/lower <frames>
```

#### Start face upper calibration

Start the upper calibration process for the face.

```osc
/snout/face/calibrate/upper <shape> <frames>
```

#### Capture frame

Capture the next processed face frame to an image file.
The path must be absolute.

```osc
/snout/face/capture <path>
```

Capture the next processed eye frame to an image file. side can be `left` or `right`.
The path must be absolute.

```osc
/snout/eye/capture <side> <path>
```

## Available face shapes

- cheekPuffLeft
- cheekPuffRight
- cheekSuckLeft
- cheekSuckRight
- jawOpen
- jawForward
- jawLeft
- jawRight
- noseSneerLeft
- noseSneerRight
- mouthFunnel
- mouthPucker
- mouthLeft
- mouthRight
- mouthRollUpper
- mouthRollLower
- mouthShrugUpper
- mouthShrugLower
- mouthClose
- mouthSmileLeft
- mouthSmileRight
- mouthFrownLeft
- mouthFrownRight
- mouthDimpleLeft
- mouthDimpleRight
- mouthUpperUpLeft
- mouthUpperUpRight
- mouthLowerDownLeft
- mouthLowerDownRight
- mouthPressLeft
- mouthPressRight
- mouthStretchLeft
- mouthStretchRight
- tongueOut
- tongueUp
- tongueDown
- tongueLeft
- tongueRight
- tongueRoll
- tongueBendDown
- tongueCurlUp
- tongueSquish
- tongueFlat
- tongueTwistLeft
- tongueTwistRight
