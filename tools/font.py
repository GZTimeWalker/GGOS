from PIL import Image, ImageDraw, ImageFont

ASCII = bytes(i for i in range(128)).decode()

class FontCfg:
    NAME = 'JBMONO'
    FONT = 'JetBrainsMono.ttf'
    SIZE = 26
    WIDTH = int(SIZE / 16 * 9) + 1
    HEIGHT = SIZE + 4
    X_PAD = 0
    Y_PAD = 0
    CHAR_SIZE = 25
    SIZE = (WIDTH * 16, HEIGHT * 6)

    def as_title(self):
        self.NAME = 'JBMONO_TITLE'
        self.FONT = 'JetBrainsMono.ttf'
        self.BLOCK_SIZE = 52
        self.WIDTH = int(self.BLOCK_SIZE / 16 * 9)
        self.HEIGHT = self.BLOCK_SIZE + 4
        self.X_PAD = 0
        self.Y_PAD = 0
        self.CHAR_SIZE = 46
        self.SIZE = (self.WIDTH * 16, self.HEIGHT * 6)

    # def as_pixel(self):
    #     self.NAME = 'FUSION_PIXEL'
    #     self.FONT = 'fusion-pixel.ttf'
    #     self.SIZE = 28
    #     self.WIDTH = int(self.SIZE / 16 * 9) + 1
    #     self.HEIGHT = self.SIZE + 4
    #     self.X_PAD = 0
    #     self.Y_PAD = 0
    #     self.CHAR_SIZE = 30
    #     self.SIZE = (self.WIDTH * 16, self.HEIGHT * 6)

    def __repr__(self):
        return f'{self.NAME}: {self.SIZE} ({self.WIDTH},{self.HEIGHT})'

FORMAT = "RGB"
BG = (0,0,0)
FG = (255,255,255)

def draw_img(cfg: FontCfg):
    im = Image.new(FORMAT, cfg.SIZE, BG)
    font = ImageFont.truetype(f"assets/font/{cfg.FONT}", size=cfg.CHAR_SIZE, index=0)
    draw = ImageDraw.Draw(im)

    draw.rectangle([(0, 0), cfg.SIZE], fill=BG)

    table = ''.join(i for i in ASCII if i.isprintable()) + '?'
    table = [table[idx * 16:idx * 16 + 16] for idx in range(6)]


    for idy, line in enumerate(table):
        for idx, ch in enumerate(line):
            draw.text((cfg.WIDTH * idx + cfg.X_PAD, cfg.HEIGHT * idy + cfg.Y_PAD), ch, font=font, fill=FG)

    return im

def get_1bit(cfg: FontCfg, im):
    charmap = []
    for y in range(cfg.SIZE[1]):
        v = 0
        bit = 0
        for x in range(cfg.SIZE[0]):
            b = im.getpixel((x, y))
            v = (v << 1) + (1 if b[0] > 35 else 0)
            bit += 1
            if bit == 8:
                charmap.append(v)
                v = 0
                bit = 0
        if bit != 0:
            charmap.append(v << (7 - bit))
    return charmap

def gen_font_border(cfg: FontCfg, im):
    draw = ImageDraw.Draw(im)
    # draw a border to identify the font size
    for y in range(cfg.SIZE[1]):
        for x in range(cfg.SIZE[0]):
            if x % cfg.WIDTH == 0 or y % cfg.HEIGHT == 0:
                draw.point((x, y), fill=(255, 0, 0))

def gen(cfg: FontCfg):
    im = draw_img(cfg)
    res = get_1bit(cfg, im)

    print(cfg, len(res))
    with open(f'pkg/kernel/assets/{cfg.NAME}.raw', 'wb') as fp:
        fp.write(bytes(res))

    gen_font_border(cfg, im)
    im.save(f'assets/img/{cfg.NAME}.png')


if __name__ == "__main__":
    cfg = FontCfg()
    gen(cfg)
    cfg.as_title()
    gen(cfg)
    # cfg.as_pixel()
    # gen(cfg)
