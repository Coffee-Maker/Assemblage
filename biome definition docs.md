## Global Parameters

> ## X
> The scenespace X position of the voxel

> ## Y
> The scenespace Y position of the voxel

> ## Z
> The scenespace Z position of the voxel

> ## Depth
> The vertical depth of the voxel

> ## Moisture
> The moisture content of the voxel (In litres). This value ranges from 0 to 1000, where 1000 is completely saturated with moisture

> ## Temperature
> The average year-round temperature of the voxel

> ## Density
> The density blended between biomes. Inferred from the current biome profiles.

<br>

---

<br>

## Functions
- Sin (n)
- Cos (n)
- Clamp (n, min, max)
- Grad(n, params...)

    ```
    An example that shows the Y value being used to generate a vertical gradient from a value of 0 at Y = 0 to a value of 1 at Y = 10

    Grad(Y, [0, 0], [10, 1])
    ```

<br>

---

<br>

## Samplers
<p>A sampler acts as an input for formulas to use later in the generation. They can also reference one another. A sampler is required to specify it's type</p>

<br>

> ## Simplex
> <p>Samples simplex noise with the given wavelength and amplitude. Required fields are:</p>
>
> - Wavelength (Float)
> - Amplitude (Float)
>
> *Example of a sampler that samples simplex noise*
> ```json
> {
>     "Name": "MyNoise",
>     "Type": "Simplex",
>     "Wavelength": 50,
>     "Amplitude": 20
> }
> ```

<br>

> ## Formula
> <p>Processes the given formula. Required fields are:</p>
>
> - Formula (String)
>
> *Example of a sampler that uses a formula
> ```json
> {
>     "Name": "MyFormula",
>     "Type": "Formula",
>     "Formula": "MyNoise - Y"
> }
> ```

<br>

---

<br>

## Required Outputs
<p> A biome profile is required to define the following fields:

- Voxel Density (Formula->Float)
- Voxel Type (Formula->String)
- Voxel Shape (Formula->Shape)