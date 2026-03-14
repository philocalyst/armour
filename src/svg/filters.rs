use svg::node::element::{
    Filter, FilterEffectComposite, FilterEffectDisplacementMap, FilterEffectDistantLight,
    FilterEffectFlood, FilterEffectGaussianBlur, FilterEffectMerge, FilterEffectMergeNode,
    FilterEffectMorphology, FilterEffectOffset, FilterEffectSpecularLighting,
    FilterEffectTurbulence,
};

pub fn create_text_outline() -> Filter {
    let morphology = FilterEffectMorphology::new()
        .set("in", "SourceAlpha".to_string())
        .set("operator", "dilate")
        .set("radius", 0.31)
        .set("result", "dilated".to_string());

    let offset = FilterEffectOffset::new()
        .set("in", "dilated".to_string())
        .set("dx", -0.21)
        .set("dy", 0.41)
        .set("result", "offsetOutline".to_string());

    let flood = FilterEffectFlood::new()
        .set("flood-color", "#FF0000")
        .set("result", "outlineColor".to_string());

    let composite = FilterEffectComposite::new()
        .set("in", "outlineColor".to_string())
        .set("in2", "offsetOutline".to_string())
        .set("operator", "in")
        .set("result", "outline".to_string());

    let merge = FilterEffectMerge::new()
        .add(FilterEffectMergeNode::new().set("in", "outline"))
        .add(FilterEffectMergeNode::new().set("in", "SourceGraphic"));

    Filter::new()
        .set("id", "outlineBehindFilter")
        .add(morphology)
        .add(offset)
        .add(flood)
        .add(composite)
        .add(merge)
}

pub fn create_nnnoise_filter(id: &str) -> Filter {
    let fe_turbulence = FilterEffectTurbulence::new()
        .set("type", "turbulence")
        .set("baseFrequency", "0.102")
        .set("numOctaves", "4")
        .set("seed", "15")
        .set("stitchTiles", "stitch")
        .set("x", "0%")
        .set("y", "0%")
        .set("width", "100%")
        .set("height", "100%")
        .set("result", "turbulence");

    let fe_distant_light = FilterEffectDistantLight::new()
        .set("azimuth", "3")
        .set("elevation", "129");

    let fe_specular_lighting = FilterEffectSpecularLighting::new()
        .set("surfaceScale", "12")
        .set("specularConstant", "0.9")
        .set("specularExponent", "20")
        .set("lighting-color", "#7957A8")
        .set("x", "0%")
        .set("y", "0%")
        .set("width", "100%")
        .set("height", "100%")
        .set("in", "turbulence")
        .set("result", "specularLighting")
        .add(fe_distant_light);

    Filter::new()
        .set("id", id)
        .set("x", "-20%")
        .set("y", "-20%")
        .set("width", "140%")
        .set("height", "140%")
        .set("filterUnits", "objectBoundingBox")
        .set("primitiveUnits", "userSpaceOnUse")
        .set("color-interpolation-filters", "linearRGB")
        .add(fe_turbulence)
        .add(fe_specular_lighting)
}

pub fn create_speckle_filter(id: &str, seed: u32, badge_height: f32) -> Filter {
    let ratio = badge_height / 800.0;
    let displacement_scale = 32.0 * ratio;
    let blur_std = 3.0 * ratio;

    let fe_turbulence = FilterEffectTurbulence::new()
        .set("type", "turbulence")
        .set("baseFrequency", "0.022 0.218")
        .set("numOctaves", "2")
        .set("seed", seed)
        .set("stitchTiles", "stitch")
        .set("x", "0%")
        .set("y", "0%")
        .set("width", "100%")
        .set("height", "100%")
        .set("result", "turbulence");

    let fe_blur = FilterEffectGaussianBlur::new()
        .set("stdDeviation", format!("0 {blur_std:.2}"))
        .set("x", "0%")
        .set("y", "0%")
        .set("width", "100%")
        .set("height", "100%")
        .set("in", "turbulence")
        .set("edgeMode", "duplicate")
        .set("result", "blur");

    let fe_displacement = FilterEffectDisplacementMap::new()
        .set("in", "SourceGraphic")
        .set("in2", "blur")
        .set("scale", format!("{displacement_scale:.2}"))
        .set("xChannelSelector", "R")
        .set("yChannelSelector", "B")
        .set("x", "0%")
        .set("y", "0%")
        .set("width", "100%")
        .set("height", "100%")
        .set("result", "displacementMap");

    Filter::new()
        .set("id", id)
        .set("x", "-20%")
        .set("y", "-20%")
        .set("width", "140%")
        .set("height", "140%")
        .set("filterUnits", "objectBoundingBox")
        .set("primitiveUnits", "userSpaceOnUse")
        .set("color-interpolation-filters", "sRGB")
        .add(fe_turbulence)
        .add(fe_blur)
        .add(fe_displacement)
}
