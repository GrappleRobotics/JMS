import { createTheme } from "@mui/material/styles";

declare module '@mui/material/styles' {
  interface Palette {
    red: Palette['primary'];
    blue: Palette['primary'];
    orange: Palette['primary'];
    purple: Palette['primary'];
    gold: Palette['primary'];
    silver: Palette['primary'];
    bronze: Palette['primary'];
  }

  interface PaletteOptions {
    red?: PaletteOptions['primary'];
    blue?: PaletteOptions['primary'];
    orange?: PaletteOptions['primary'];
    purple?: PaletteOptions['primary'];
    gold?: PaletteOptions['primary'];
    silver?: PaletteOptions['primary'];
    bronze?: PaletteOptions['primary'];
  }
}

declare module '@mui/material/Button' {
  interface ButtonPropsColorOverrides {
    red: true;
    blue: true;
    orange: true;
    purple: true;
  }
}

let jmsBaseTheme = createTheme({
  palette: {
    mode: "dark"
  }
});

export const jmsTheme = createTheme(jmsBaseTheme, {
  palette: {
    red: jmsBaseTheme.palette.augmentColor({ color: { main: "#f53e31" }, name: "red" }),
    blue: jmsBaseTheme.palette.augmentColor({ color: { main: "#3152f5" }, name: "blue" }),
    orange: jmsBaseTheme.palette.augmentColor({ color: { main: "#FF9800" }, name: "orange" }),
    purple: jmsBaseTheme.palette.augmentColor({ color: { main: "#9C27B0" }, name: "purple" }),
    gold: jmsBaseTheme.palette.augmentColor({ color: { main: "#9c8501" }, name: "gold" }),
    silver: jmsBaseTheme.palette.augmentColor({ color: { main: "#888" }, name: "silver" }),
    bronze: jmsBaseTheme.palette.augmentColor({ color: { main: "#865320" }, name: "bronze" }),
  },
});