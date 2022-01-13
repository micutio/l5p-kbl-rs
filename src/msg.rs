pub const HELP_GENERAL: &str = r#"
Lenovo Legion 5 Pro 2021 keyboard light controller
Inspired by https://github.com/imShara/l5p-kbl/
2022 Michael Wagner

Supported LED modes:
off                             Turn all keyboard backlight off.
static                          Show static coloured light for each zone.
breath                          Fade light in and out.
wave                            Directed left or right rainbow animation.
hue                             Continuously cycle between hues.

USAGE:
    l5p-kbl-rs [set | monitor] 

    ---------------------------------------------------------------------------

    set: directly set led mode and attributes

    monitor: assign keyboard LED configurations to changes in system variables
    "#;

pub const HELP_SET: &str = r#"
Lenovo Legion 5 Pro 2021 keyboard light controller
Inspired by https://github.com/imShara/l5p-kbl/
2022 Michael Wagner

l5p_kbl-set: directly set led mode and attributes

USAGE:
    l5p-kbl-rs set mode [options] colour1 [colour2] [colour3] [colour4]

    Colours are given as RGB tripels in hexadecimal form, e.g.: FF00ed,
    corresponding to each of the four regions on the keyboard, from left to
    right. If less than four colors are given, then the last colour is repeated
    for the remaining areas.

    Supported LED modes:
    off                             Turn all keyboard backlight off.
    static                          Show static coloured light for each zone.
    breath                          Fade light in and out.
    wave                            Directed left or right rainbow animation.
    hue                             Continuously cycle between hues.

OPTIONS:
        
    -b | --brightness <[1,2]>   Brightness: 1 = dimmer, 2 = brighter
                                Available to all modes.

    -s | --speed <[1..4]>       Animation frequency: 1 = slower, 4 = faster
                                Only applies to modes: breath | wave | hue
    
    -d | --dir 'ltr' | 'rtl'    Set wave animation to go from left to right
                                or right to left. Only applies to mode wave
    "#;

pub const HELP_MONITOR: &str = r#"
Lenovo Legion 5 Pro 2021 keyboard light controller
Inspired by https://github.com/imShara/l5p-kbl/
2022 Michael Wagner

l5p_kbl-monitor: map keyboard LED settings to changes in system variables

USAGE:
    l5p-kbl-rs monitor -f | --file <filepath>
    
OPTIONS:
    -f | --file <filepath>      read variable to led config mapping from
                                provided JSON file                

    "#;
