import { IJsonModel } from "flexlayout-react";

export const defaultLayoutModel: IJsonModel = {
  global: {
    tabEnableClose: true,
    tabEnableRename: false,
    tabSetEnableMaximize: false,
    tabSetTabLocation: "top",
  },
  layout: {
    type: "row",
    weight: 100,
    children: [
      {
        type: "tabset",
        weight: 20,
        children: [{ type: "tab", name: "Properties", component: "properties" }]
      },
      {
        type: "tabset",
        weight: 60,
        enableDrop: false,
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
        type: "tabset",
        weight: 20,
        children: [
          { type: "tab", name: "Layers", component: "right-sidebar" }
        ]
      }
    ]
  }
};
