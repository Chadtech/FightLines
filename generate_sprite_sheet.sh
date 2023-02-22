# Make blue units
/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  ./shared/src/sprites/units/tank_red.aseprite \
  --palette ./shared/src/sprites/palette_blue.aseprite \
  --save-as ./shared/src/sprites/units/tank_blue.aseprite

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  ./shared/src/sprites/units/infantry_red.aseprite \
  --palette ./shared/src/sprites/palette_blue.aseprite \
  --save-as ./shared/src/sprites/units/infantry_blue.aseprite

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  ./shared/src/sprites/units/truck_red.aseprite \
  --palette ./shared/src/sprites/palette_blue.aseprite \
  --save-as ./shared/src/sprites/units/truck_blue.aseprite

# Make moved units
/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  ./shared/src/sprites/units/infantry_red.aseprite \
  --palette ./shared/src/sprites/palette_red_moved.aseprite \
  --save-as ./shared/src/sprites/units/infantry_red_moved.aseprite

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  ./shared/src/sprites/units/tank_red.aseprite \
  --palette ./shared/src/sprites/palette_red_moved.aseprite \
  --save-as ./shared/src/sprites/units/tank_red_moved.aseprite

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  ./shared/src/sprites/units/truck_red.aseprite \
  --palette ./shared/src/sprites/palette_red_moved.aseprite \
  --save-as ./shared/src/sprites/units/truck_red_moved.aseprite

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  ./shared/src/sprites/units/infantry_blue.aseprite \
  --palette ./shared/src/sprites/palette_blue_moved.aseprite \
  --save-as ./shared/src/sprites/units/infantry_blue_moved.aseprite

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  ./shared/src/sprites/units/tank_blue.aseprite \
  --palette ./shared/src/sprites/palette_blue_moved.aseprite \
  --save-as ./shared/src/sprites/units/tank_blue_moved.aseprite

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  ./shared/src/sprites/units/truck_blue.aseprite \
  --palette ./shared/src/sprites/palette_blue_moved.aseprite \
  --save-as ./shared/src/sprites/units/truck_blue_moved.aseprite

# Make Unit Thumbnails
/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  --oneframe \
  ./shared/src/sprites/units/infantry_red.aseprite \
  --save-as ./shared/src/sprites/units/infantry_red.png
cp ./shared/src/sprites/units/infantry_red1.png ./server/src/assets/infantry_red.png

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  --oneframe \
  ./shared/src/sprites/units/tank_red.aseprite \
  --save-as ./shared/src/sprites/units/tank_red.png
cp ./shared/src/sprites/units/tank_red1.png ./server/src/assets/tank_red.png

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  --oneframe \
  ./shared/src/sprites/units/truck_red.aseprite \
  --save-as ./shared/src/sprites/units/truck_red.png
cp ./shared/src/sprites/units/truck_red1.png ./server/src/assets/truck_red.png

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  --oneframe \
  ./shared/src/sprites/units/infantry_blue.aseprite \
  --save-as ./shared/src/sprites/units/infantry_blue.png
cp ./shared/src/sprites/units/infantry_blue1.png ./server/src/assets/infantry_blue.png

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  --oneframe \
  ./shared/src/sprites/units/tank_blue.aseprite \
  --save-as ./shared/src/sprites/units/tank_blue.png
cp ./shared/src/sprites/units/tank_blue1.png ./server/src/assets/tank_blue.png

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  -b \
  --oneframe \
  ./shared/src/sprites/units/truck_blue.aseprite \
  --save-as ./shared/src/sprites/units/truck_blue.png

# Units
/Applications/Aseprite.app/Contents/MacOS/aseprite \
  --batch \
    ./shared/src/sprites/units/infantry_red.aseprite \
    ./shared/src/sprites/units/infantry_blue.aseprite \
    ./shared/src/sprites/units/tank_red.aseprite \
    ./shared/src/sprites/units/tank_blue.aseprite \
    ./shared/src/sprites/units/truck_red.aseprite \
    ./shared/src/sprites/units/truck_blue.aseprite \
  --sheet ./shared/src/sprites/units_sheet.png

# Moved Units
/Applications/Aseprite.app/Contents/MacOS/aseprite \
  --batch \
    ./shared/src/sprites/units/infantry_red_moved.aseprite \
    ./shared/src/sprites/units/infantry_blue_moved.aseprite \
    ./shared/src/sprites/units/tank_red_moved.aseprite \
    ./shared/src/sprites/units/tank_blue_moved.aseprite \
    ./shared/src/sprites/units/truck_red_moved.aseprite \
    ./shared/src/sprites/units/truck_blue_moved.aseprite \
  --sheet ./shared/src/sprites/unit_moved_sheet.png

# Flags
/Applications/Aseprite.app/Contents/MacOS/aseprite \
  --batch \
    ./shared/src/sprites/many_red.aseprite \
    ./shared/src/sprites/many_blue.aseprite \
  --sheet ./shared/src/sprites/flags_sheet.png

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
    ./shared/src/sprites/arrow_right_moved.aseprite \
    ./shared/src/sprites/arrow_x_moved.aseprite \
    ./shared/src/sprites/arrow_left_moved.aseprite \
    ./shared/src/sprites/arrow_up_moved.aseprite \
    ./shared/src/sprites/arrow_y_moved.aseprite \
    ./shared/src/sprites/arrow_down_moved.aseprite \
    ./shared/src/sprites/arrow_right_turn_up_moved.aseprite \
    ./shared/src/sprites/arrow_right_turn_down_moved.aseprite \
    ./shared/src/sprites/arrow_left_turn_up_moved.aseprite \
    ./shared/src/sprites/arrow_left_turn_down_moved.aseprite \
    ./shared/src/sprites/hills_tile.aseprite \
    ./shared/src/sprites/forest_tile.aseprite \
  --sheet ./shared/src/sprites/misc_sheet.png

/Applications/Aseprite.app/Contents/MacOS/aseprite \
  --batch \
    ./shared/src/sprites/units_sheet.png \
    ./shared/src/sprites/unit_moved_sheet.png \
    ./shared/src/sprites/misc_sheet.png \
    ./shared/src/sprites/flags_sheet.png \
  --sheet ./server/src/assets/sheet.png \
  --sheet-type columns

cargo run sprites