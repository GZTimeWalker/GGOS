from PIL import Image, ImageDraw, ImageFont

SIZE = 26

WIDTH = int(SIZE / 16 * 9) + 1
HEIGHT = SIZE + 4

X_PAD = 0
Y_PAD = 0
CHAR_SIZE = 25

size = (WIDTH * 16, HEIGHT * 6)

print(size, WIDTH, HEIGHT)

FORMAT = "RGB"
BG = (0,0,0)
FG = (255,255,255)

ASCII = bytes(i for i in range(128)).decode()
im = Image.new(FORMAT, size, BG)
font = ImageFont.truetype(r"assets/JetBrainsMono.ttf", size=CHAR_SIZE, index=0)
draw = ImageDraw.Draw(im)

draw.rectangle([(0, 0), size], fill=BG)
x, y = 0, 0

table = ''.join(i for i in ASCII if i.isprintable()) + '?'
table = [table[idx * 16:idx * 16 + 16] for idx in range(6)]


for idy, line in enumerate(table):
    for idx, ch in enumerate(line):
        draw.text((WIDTH * idx + X_PAD, HEIGHT * idy + Y_PAD), ch, font=font, fill=FG)

def get_1bit(im):
    charmap = []
    for y in range(size[1]):
        v = 0
        bit = 0
        for x in range(size[0]):
            b = im.getpixel((x, y))
            v = (v << 1) + (1 if b[0] > 32 else 0)
            bit += 1
            if bit == 8:
                charmap.append(v)
                v = 0
                bit = 0
        if bit != 0:
            charmap.append(v << (7 - bit))
    return charmap

res = get_1bit(im)
print(len(res))

with open('pkg/kernel/src/assets/font.raw', 'wb') as fp:
    fp.write(bytes(res))
