for size in 16 32 48 64 128 256 512; do
	convert assets/icon_1024.png -resize ${size}x${size} assets/icon_${size}.png;
done
