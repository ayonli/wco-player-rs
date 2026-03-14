for size in 16 32 48 64 128 256 512; do
	convert icons/icon_1024.png -resize ${size}x${size} icons/icon_${size}.png;
done
