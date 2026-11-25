import { createElement, type ReactElement, type SVGProps } from "react";

import { GLYPHS, type Glyph } from "./glyphs";

type Weight = "regular" | "bold";

export interface IconProps extends Omit<SVGProps<SVGSVGElement>, "ref"> {
  size?: number;
  weight?: Weight;
}

export type IconComponent = (props: IconProps) => ReactElement;

function makeIcon(glyphs: Partial<Record<Weight, Glyph>>): IconComponent {
  return function Icon({ size = 24, weight = "regular", ...rest }: IconProps) {
    const glyph = glyphs[weight] ?? glyphs.regular ?? glyphs.bold ?? [];
    return (
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width={size} height={size} fill="currentColor" {...rest}>
        {glyph.map(([tag, attributes], index) => createElement(tag, { key: index, ...attributes }))}
      </svg>
    );
  };
}

export const CaretDownIcon = makeIcon(GLYPHS.CaretDown);
export const CheckCircleIcon = makeIcon(GLYPHS.CheckCircle);
export const CheckIcon = makeIcon(GLYPHS.Check);
export const CopyIcon = makeIcon(GLYPHS.Copy);
export const CubeIcon = makeIcon(GLYPHS.Cube);
export const DesktopIcon = makeIcon(GLYPHS.Desktop);
export const GearSixIcon = makeIcon(GLYPHS.GearSix);
export const HardDrivesIcon = makeIcon(GLYPHS.HardDrives);
export const InfoIcon = makeIcon(GLYPHS.Info);
export const MoonIcon = makeIcon(GLYPHS.Moon);
export const SignOutIcon = makeIcon(GLYPHS.SignOut);
export const SunIcon = makeIcon(GLYPHS.Sun);
export const WarningCircleIcon = makeIcon(GLYPHS.WarningCircle);
export const WarningIcon = makeIcon(GLYPHS.Warning);
