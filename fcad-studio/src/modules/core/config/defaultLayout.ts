import { IJsonModel } from "flexlayout-react";

export const defaultLayoutModel: IJsonModel = {
  global: {
    tabEnableClose: true,
    tabEnableRename: false,
    tabSetEnableMaximize: false,
    tabSetTabLocation: "top",
    tabSetEnableDrop: true,
  },
  layout: {
    type: "row",
    weight: 100,
    children: [
      {
        type: "tabset",
        weight: 20,
        enableDrop: true,
        children: [{ type: "tab", name: "Properties", component: "properties" }]
      },
      {
        type: "tabset",
        weight: 60,
        enableDrop: true,
        enableDrag: false,
        children: [
          { 
            type: "tab", 
            name: "Canvas", 
            component: "canvas", 
            enableClose: false, 
            enableDrag: false 
          }
        ]
      },
      {
        type: "row",
        weight: 20,
        children: [
          {
            type: "tabset",
            weight: 50,
            enableDrop: true,
            children: [
              { type: "tab", name: "Layers", component: "layers" },
              { type: "tab", name: "Assets", component: "assets" }
            ]
          },
          {
            type: "tabset",
            weight: 50,
            enableDrop: true,
            children: [
              { type: "tab", name: "AI Console", component: "ai" }
            ]
          }
        ]
      }
    ]
  }
};
