# Snout

This is a rust implementation of Project Babble's baballonia face tracking sofware.
It's designed to be a library; easy to integrate in a variety of frontend projects. However it can also be used as a CLI application through snout-cli.

- Building:
  - [Building and running the cli](#building-and-running-the-cli)
  - [Installing on NixOS](#installing-on-nixos)
- Configuring:
  - See [configuring.md](docs/configuring.md)
- Usage:
  - [Tracking](#tracking)
  - [Training an eye model](#training-an-eye-model)
    - [Sampling data](#sampling-training-data)
    - [Training the model](#training-the-model)
  - [Troubleshooting](#troubleshooting)
- Remote Control:
  - See [remote.md](docs/remote.md)
- Contributing:
  - See [contributors.md](docs/contributors.md)

## Required dependencies

Snout requires the following build dependencies (in the form of fedora package names):

- llvm
- llvm-devel
- onnxruntime
- onnxruntime-devel
- rust

## Building and running the CLI

Clone the repository,

```sh
git clone https://github.com/Darksecond/libsnout.git
```

and then build the program.

```sh
cd libsnout
cargo build --release -p snout-cli
```

The snout-cli executable will be located under `target/release/`

snout-cli can either be executed from this directory, like so:

```sh
target/release/snout-cli
```

or snout-cli can be added to your `$PATH` and executed like so:

```sh
snout-cli
```

These docs will assume that snout-cli is in your `$PATH`. If this is not the case, replace `snout-cli` in the following commands with the path to snout-cli. Like shown above.

Help on how to use the cli tool can be obtained with:

```sh
snout-cli help
```

### Installing on NixOS

Add Snout to your flake.nix inputs:

```nix
  libsnout.url = "github:Darksecond/libsnout";

```

Either use the package directly or add it to your overlays:

```nix
nixpkgs.overlay = [
  (final: prev: {
    snout-cli = libsnout.packages."${pkgs.stdenv.hostPlatform.system}".default;
  })
];

environment.systemPackages = with pkgs; [
  snout-cli
];

```

## Tracking

> [!IMPORTANT]
> Before being able to use Snout for face/eye tracking and training models, one must configure it. See [configuring.md](docs/configuring.md) for information on how to do so.

Snout comes with a working face tracking model. It's the same as in the baballonia repository, but ran through `onnxsim`.
Make sure to download it from this repository and reference it in your configuration file if you plan to use face tracking.

Once you have set up your configuration file to point to your cameras, and set the output OSC destination to the correct values for your program of choice. You can start tracking with the following command:

```sh
snout-cli track
```

This will start recording, along with sending data to the OSC endpoint specified in the configuration file.

## Training an eye model

### Sampling training data

> [!NOTE]
> To start sampling data, the configuration file must include the path to Baballonias calibration overlay. See [CONFIGURING.md](docs/configuring.md#sampling-overlay)

Training data can be obtained through the `sample` command.
The sample command will generate a directory of .bin files used during training

```sh
snout-cli sample -o my_training_data
```

The above command will output the .bin files to the "my_training_data" folder.

### Training the model

Eye models can be trained with the following command:

```sh
snout-cli train <capture> <output.onnx>
```

The `<capture>` argument can be either the directory of .bin files created by the previous sampling step, or a singular .bin file.
The resulting eye tracking model will be written to the `<output.onnx>` file.

## Troubleshooting

A camera frame can be captured and written to a file with the following command to help with debugging tracking issues, along with aligning your face:  

```sh
snout-cli capture <SOURCE> <OUTPUT.jpeg>
```

`<SOURCE>` can be any of the following camera sources `left-eye`, `right-eye`, `face`,

`<OUTPUT.jpeg>` will be the name of the file that the camera frame gets written to.

## License

Right now it's licensed under the same license as Baballonia from Project Babble is, considering this is a derivative work.
