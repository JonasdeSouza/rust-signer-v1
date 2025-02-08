mipidsi::Display
{
    SpiInterface
    {
        '_, SpiDeviceDriver
        {
            '_, SpiDriver
            {
                '_
            }
        }, 
        PinDriver
        {
            '_, PeripheralRef
            {
                '_, AnyIOPin
            }, 
            InputOutput
        }
    }, 
    ST7789, 
    PinDriver
    {
        '_, PeripheralRef
            {
                '_, AnyIOPin
            }, InputOutput
        }
    }, 
    InputOutput
}}

