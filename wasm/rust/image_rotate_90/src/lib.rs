use std::io::Cursor;

#[no_mangle]
fn main(base64: &String, clockwise: bool) -> Box<String> {
    //decode base64 to bytes
    let bytes: Vec<u8> = base64::decode(base64).unwrap();
    //get a DynamicImage from the bytes
    let original_image: image::DynamicImage = image::load_from_memory(&bytes[..]).unwrap();
    //rotate
    let new_image = if clockwise {
        original_image.rotate90()
    } else {
        original_image.rotate270()
    };
    //convert new image to base 64 and return
    Box::new(dynamic_image_to_base64(&new_image))
}

fn dynamic_image_to_base64(img: &image::DynamicImage) -> String {
    let mut image_data: Vec<u8> = Vec::new();
    img.write_to(
        &mut Cursor::new(&mut image_data),
        image::ImageOutputFormat::Png,
    )
    .unwrap();

    base64::encode(image_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    static INPUT_BASE64: &str = "iVBORw0KGgoAAAANSUhEUgAAABYAAAAWCAMAAADzapwJAAABsFBMVEVHcEz//bXvoBPKbAXAYQj+/tz/vi/WfhDfhAr5qBT/wy++Wgu2IBz2phX/53D/uCPxhgDTdgX/wCjNbwXqmxH7rBbTdwbGZwj/2VP9shjNbwbufwTRdQX/zz7//8vTeQf0kQDggQW2WArrhQTkhQbkhwbsgwH8wBXFZw3KbAX0phD2jwDPcwbHagb/7oX+51b/+qH3kwD+9HHxgwD2jwD/00z+9pD+83H+5VX5mQf7sAzGaQXMcAXQdgb7qQPpgAL6sAb8ySL8yiX5rAL/tB3/3C/pjAn/933umg//1zP/4TL/xDT90TD++JL+sBj8rBP/zS///3r+yC7/vSj/+Zz/+qz9pg3+0TX+xzKgahjAkh//zTT/6zf/0i/xuDX/3TP/0jrjhQeUWAb/40P/0jb/yinvlQz/+Ij/uSL//bv/2Dimbxr/9EvJoSDqlg7+9mHpyi//1DzpsC/w2T+eYwnRuD7//GL+0Cv/9Fv/7mv/84v/vxz/3lH+1j/2ngv//83VnSWxfyTy2zW8hiD56kffqjPZw0XfqTH42DSITgWXYg7eqDGCRgHfyEb/uBX/3UlPCgIZAAAARHRSTlMA/vqCIvz+AQL9/hIH/P39bMf9u/v+91H9/Jo36fz8+FLWF8Dp+kDEDI/7l9dj+/77kP5jw/z9/v7p12+95MOQndbXkJOrxpgAAAGUSURBVBjTRZFXU8JAFEYXiIFQBQ3YECv2rtj7EjAJCETpHUQQ26ggKPbe/7IbFTwP3+ych7tz7wcAj4QPubzyBGVrXO1aXulaapH/ewloXEhaBFdXGdvXTAsA4j9b/Wlprvqh2bY18uuRPT8TbG4KMhk+h0N9vEYT3ErTtsm2gbCZts8Gz8d+5o9HlcVJp9OCcDpHi8q6aRmyRvdEaarwkcsfHOQfc4WhUh2sRnoR6i8Sidf3493d+/jLceJCz7Uj3Q6ljrXnk/hhMHiLMsJ60x0yIG+1a1yn5j0/TdPBPb/5dEtj78SQXuekjlCENiPoSMjh4taRBmq3SupysOwagmUDLqnK3dEGQMNlPcd4AztJj8ezk/QyeP2lGn2pi/VCnPFfe6PRwNFdGIdESsQv328lIB6+8WWzvuwDDkmrFuOX18VqCbuKCT+9HTGcnRRSIn55CeihahUkCdNpSJKEkKopH7whti9UEAiFcJ+q6f49OIqBJiplRaQorUhc6UcMZLNz8wZDk1qElcsp9yrDsLZKxd9gIFo8ETqejgAAAABJRU5ErkJggg==";
    #[test]
    fn rotate_image_90_clockwise_works() {
        let output_clockwise = "iVBORw0KGgoAAAANSUhEUgAAABYAAAAWCAYAAADEtGw7AAAE60lEQVR4AZWUW2yURRTH/2fmu+1+2+52e6GVRrmqJdEYNVgCWsEYUBsuxpgYY2LCgzyIRSNRnzDBEG9JJfpAlAcTIw+GRB5AAyKgQhoxKqZVUAhgAKGttNvL3r7LjHO+0kKrL36bM3PmnDO/PXPmQis3dWLmt6PhkNjw9wp1o71rzwmXx9vX3lXlflL+K5Z9/wJPBnJ/+YJa43rqEcfVd9T4yPIE39IDKlQnpNR7nr762BG2cezMRKbA3bVH6cXRZZoDt5QPrktl1LaGPN1e32LB9QS040BYNsgIx6jRAqr9Y0crBevlp0ZWfT8TPgXmYJat0f7ulhaxyW/04GQoSmU8Eo5PsG0BCQXpEjJZBasW0ENS9fVi/PzYxicHOj+4ES4Yxtlyv00f+PCWOdYmtykd+bW28mtrLOHnJPwMwdRDe45AGqRdMjZbonGxEsvXq9olre9/1rT3eS4Hw5lF595IJxv1enBgfets2snQbI0UTiZlIHkNxyZtBSCboGUEcj0gk4MWeU2pJtLeQkVxKHDsbQz3DLdPliWBvjJwOJ/29Ta3zkXGt8j2rOtQqRIobIAyFqpE4I+yDmnb0SQqJnaRRscG1CzS77KPMxeseNngmeZZVpOdtkPbIUGer0YDh775aRBkVq8sA/MtHD82gl3v/IodW37A+T7jy6RIK08hLhO81bHVds+yT+v3PcjMBOy4WO2kJFxHJGNk6sSRnmE0+g5ggODP0iiPaJSKMQYvjmG0UGErICbmaF0HLFyJmjZayw6Ly2C10m3a7IUUmizPo9FxwvDgONpW3QQECsKRQBihY0092tpzcLM5ZOfPByoSJNKATGmGwWnXsqH+XtYt4cY5KamBB4nYNoYKkdkzAfI9aB2CEsdE0zTXbF4qAx1o0JTDmdBEMyFVk+8yt1RMhE9vtZ4+/l8jksmfiFJZFFWkRqYmhyF8sx9BqMzygylzoqRtjBUUglIEupZkYodJnxU1BJQKV/k9EdubV/QLwlkKY8SKdFSp6qYGOynFwAVTBldCBTFg9uD4F/3Y/WYfPnrhMP7ouQB4MbQuITkVBqzDk1CFkd+NCsFNuYSvgnKMaqBMmsZSHceSpS3o/c0sxGyesQARTZ2K8ZJGpXjtkVMTc4iGQZe+RPBnsI/jLW5KRfnxwGC8eVY69JRHSo8VaE6Dq/O1jaSLkTkVJqwYouPR3PVTMbsVulg2x6gi+FToao+Me7879W136z6YAycPdl4Ur6qHhu6vnnGzPjqUY0e2RULo0NxevlkWaQQg89NRhEzOgeeloSNXE3kE5AFBMf3ynij3/fXcS80P9PF7IU/duTI5A6tzV44MDVSX+na8MLasyBIqgUMYAENhOCqGUsKUxTEWl0B5BTKFPrvTqhzr2/742c5uhvKVlgvabwW/bvwWL61c+rxUChYzPDJTJCmznWa5CqDYsAwX2iEiJ4bMANVR4MweWe45vWPduc6NMN/e0twk0QS8v3oz+J82h8urX9OCT+4eOi3L49F9KUfbCKtChiVSYRUUhEQma1QrAv3nKej9ebB4crjricudWw0zSZBZrE976CczZ8drIwfnpf342VQaD9dlaR4EzGNgPArDLtRJh6LdP/b5u95qWj7E84wHvGruWaaB2cDC2XOdWGfpunJoVjqlfNZVVRYYxjrLzFi2sfwDIiws/AgEq5wAAAAASUVORK5CYII=".to_string();
        assert_eq!(*main(&INPUT_BASE64.to_string(), true), output_clockwise);
    }

    #[test]
    fn rotate_image_90_anticlockwise_works() {
        let output_anticlockwise = "iVBORw0KGgoAAAANSUhEUgAAABYAAAAWCAYAAADEtGw7AAAFAklEQVR4AZVUW4iVVRT+1v5v5zZXnWPpTM6kjRc0BQuVLt4dShGLHqSXikAmSCYhSSILfAiiMO0loZdeIkhKjekiiaJoF5IQI8co1PLUmebMnDPn8t//f+/2PmdmnNSXfvbea/9rr/2tb6+91qa+l7bi1u/wzFOsf3Q9n9S/MnK6nVlxq/p3XGYfumv9P2qu+q22Sqe6robJ/m7zOVJzBarAViyxnw6E/pTfwRaBoU2tgfPSO+UTV10H3zi29mE/Nl5VerV3d+VhoeaqTzGevvDJjMHnE1nrTXNONotkGogCCInEoxAUBPA9Dt8l5P4MPNcWb+/T+15XYNPZTwGrBdWPzh18P7m8sx+zVzZC4RSE8Goa+Q5HLHmHIeeBLSInIrsSsrASIHcjOul5iSf2J9fUJgkyBaY8KXls3uDB5IZl/Vj2QoTmLsDgTFikiZT0kSIm0iaQzhDLzGJmJsma2i1hzkyFnV36xkTCOzoJqrC0k1tzrF9e1Kc9g0+mVi06gCV7IghPBxVJkCfI8Ik0GYo4BpEGAUFEJoFZgomQdMZZrOtR2ojvK+R9dprNP62Ialfu7xMDxy5avTv4EXpo70zSO+UFDjdALY/KxXFUSg4yLQyQxAkmhOCoOgZZCVNQ6Ch35EeQ0YpXrihe+3gv31BkkN+ju3Nb9KWPLCRrdYzYJcETnNJJunTuBg7v+RZHDgzhzJfjMgw6uC4R0jp+/KmA66M6UVMrmM7ISBlhtkNLpNLxsxJSXoYczbnmFjHnMcmkTf7JxljdYSJtIZMipNIaki0E6BPZZDIsXdyC787nASsjN0hhMmYmpV0Km5RCVwNrbVlAxiLUt2lJQUKieRp6V3ehe8E6+OUimloZUPbATB3Cj5HtMhCEHCOjIbIJi3gkT2po4AL3DgyfmsUGZHyRap0B1q58yG5KalLIJgIBM6U3QJ1QaqY1w4RpMNgeAwxjakGGpSWV5GlW14i4TrY+/58DTdH470Z2aPtyH261CD48AS5pTtiorCr/VcPINW9C0xBEBGF79VC0t8hohjdPE8dilPvaOFOm8ejYBQTfN3yrrBAOkIhx6eTveG/X2UZWHB8DVAyDGJCXN3S5iraODJozApHni5jLDA1jWf3i17ey6xrpVh0Sx/DbCRCVlB+Zr1xmLNDcmkBHZ9PNrIgavmFHKNgB1q6WWVQr1W39QFa6GyPw8bkC0eav6sVnbu/1HYmzG9m8bDeMHpkVfzPhjYu2WaAHVrVg8YPNmNstL0g+FxQBIiB035WBhVAIu8QCj3OvGujDuWCkWjB3nk/3uEyVn/JQvUwv48xhwLlMgic4hQGJciPelhAQNYkYStBQgGIGGWABp0ihF/GaHQm/5MOx6VUVBoVZL2k1ea60KbetcnHM0oYep6bZArEmyCsx4dgg+WRCsiSfBHGSoC7BrVBQc3m5GnNR9vR8nn+wT9+8Xz1Eu0prRD0Ug06PUODPFPp+2Fa6MGaM/7KFMjKgoRVT6AvYPlEo2UeSqucJ7lW5U3WkiDW34LJcLj74mtb3ojr5Cf8eJVAHVrPp4FtrV75G/o+Fmp3vpkBWgHzVheOQRANqFfIrPiveCFg+F14ZK7Cdb1ibDyoMxXYS+LaHXjFXz6gy/GjGF2vjmLYzgy23I8pCflUb5cCnn32PfXV3FzuubKfvkSb1dhuw0t7JcECVvlw8pApKysl2J1u19i+OgY5RJJD45AAAAABJRU5ErkJggg==".to_string();
        assert_eq!(
            *main(&INPUT_BASE64.to_string(), false),
            output_anticlockwise
        );
    }
}
