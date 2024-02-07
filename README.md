# ![icon](https://raw.github.com/stynh/ImageFlipper/master/icon.png)
Example usage on a folder:
```
./ImageFlipper.exe --folder "C:/MyFolder" --from "jpg" --to "png"
```

Converting all images that are not the target format:
```
./ImageFlipper.exe --folder "C:/MyFolder" --all --to "png"
```

Or for a single file:
```
./ImageFlipper.exe --file "C:/MyFolder/image.png" --to "webp"
```

You can also specify an output folder (created if it does not exist):
```
./ImageFlipper.exe --folder "C:/MyFolder" --all --to "png" --output "./converted"
```
