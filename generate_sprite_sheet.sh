# Units
/Applications/Aseprite.app/Contents/MacOS/aseprite \
  --batch \
    ./shared/src/sprites/infantry_red.aseprite \
    ./shared/src/sprites/infantry_blue.aseprite \
    ./shared/src/sprites/tank_red.aseprite \
  --sheet ./shared/src/sprites/units_sheet.png

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
    ./shared/src/sprites/arrow_left.aseprite \
    ./shared/src/sprites/arrow_up.aseprite \
    ./shared/src/sprites/arrow_y.aseprite \
    ./shared/src/sprites/arrow_down.aseprite \
    ./shared/src/sprites/arrow_right_turn_up.aseprite \
    ./shared/src/sprites/arrow_right_turn_down.aseprite \
    ./shared/src/sprites/arrow_left_turn_up.aseprite \
    ./shared/src/sprites/arrow_left_turn_down.aseprite \
  --sheet ./shared/src/sprites/misc_sheet.png

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  --batch \
    ./shared/src/sprites/units_sheet.png \
    ./shared/src/sprites/unit_moved_sheet.png \
    ./shared/src/sprites/misc_sheet.png \
  --sheet ./server/src/assets/sheet.png \
  --sheet-type columns

cargo run sprites