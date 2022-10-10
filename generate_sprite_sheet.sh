# Units
/Applications/Aseprite.app/Contents/MacOS/aseprite \
  --batch \
    ./shared/src/sprites/infantry_red.aseprite \
    ./shared/src/sprites/infantry_blue.aseprite \
  --sheet ./shared/src/sprites/unit_sheet.png

# Moved Units
/Applications/Aseprite.app/Contents/MacOS/aseprite \
  --batch \
    ./shared/src/sprites/infantry_red_moved.aseprite \
    ./shared/src/sprites/infantry_blue_moved.aseprite \
  --sheet ./shared/src/sprites/unit_moved_sheet.png

# Misc
/Applications/Aseprite.app/Contents/MacOS/aseprite \
  --batch \
    ./shared/src/sprites/grass_tile.aseprite \
    ./shared/src/sprites/not_visible.aseprite \
    ./shared/src/sprites/unit_outline.aseprite \
    ./shared/src/sprites/move_option.aseprite \
    ./shared/src/sprites/arrow_right.aseprite \
    ./shared/src/sprites/arrow_x.aseprite \
  --sheet ./shared/src/sprites/misc_sheet.png

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  --batch \
    ./shared/src/sprites/units_sheet.png \
    ./shared/src/sprites/unit_moved_sheet.png \
    ./shared/src/sprites/misc_sheet.png \
  --sheet ./server/src/assets/sheet.png \
  --sheet-type columns

cargo run sprites