declare module "*.svg" {
  import type { Component, JSX } from "solid-js";
  const SVGComponent: Component<JSX.SvgSVGAttributes<SVGSVGElement>>;
  export default SVGComponent;
}
