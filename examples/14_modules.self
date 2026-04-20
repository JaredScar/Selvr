// 14 — Module system
// import / export work like TypeScript ES modules.

// Exporting from a module file (math.vx would look like this):
export const PI: f64 = 3.14159265358979;
export const E:  f64 = 2.71828182845904;

export fn degToRad(degrees: f64): f64 {
    return degrees * PI / 180.0;
}

export fn radToDeg(radians: f64): f64 {
    return radians * 180.0 / PI;
}

// Nested module — like a TypeScript namespace
mod trig {
    export fn sin(x: f64): f64 { return x.sin(); }
    export fn cos(x: f64): f64 { return x.cos(); }
    export fn tan(x: f64): f64 { return x.tan(); }
}

mod stats {
    export fn mean(xs: f64[]): f64 {
        if xs.length === 0 { return 0.0; }
        return xs.reduce((acc, x) => acc + x, 0.0) / xs.length as f64;
    }

    export fn variance(xs: f64[]): f64 {
        const m = mean(xs);
        return xs.map((x) => (x - m) * (x - m))
                 .reduce((acc, x) => acc + x, 0.0)
                 / xs.length as f64;
    }

    export fn stdDev(xs: f64[]): f64 {
        return Math.sqrt(variance(xs));
    }
}

// Importing — identical to TypeScript ES module imports
import { PI, degToRad } from "./math";
import { sin, cos }     from "./math/trig";
import { mean, stdDev } from "./math/stats";

fn main(): void {
    console.log(`PI = ${PI}`);
    console.log(`90° in radians = ${degToRad(90.0).toFixed(4)}`);

    const angle = degToRad(45.0);
    console.log(`sin(45°) = ${sin(angle).toFixed(4)}`);
    console.log(`cos(45°) = ${cos(angle).toFixed(4)}`);

    const data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    console.log(`mean    = ${mean(data).toFixed(2)}`);    // 5.0
    console.log(`stdDev  = ${stdDev(data).toFixed(2)}`);  // 2.0
}
