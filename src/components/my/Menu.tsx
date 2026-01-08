import {
  Menubar,
  MenubarCheckboxItem,
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarShortcut,
  MenubarTrigger,
} from "@/components/ui/menubar"
import "./Menu.css";
import { useContext } from "react"
import { useHotkeys } from 'react-hotkeys-hook'
import { ThemeProviderState, useTheme } from "../theme-provider";
import { AppContext } from "@/lib/AppContext";

function toggleDarkMode(themeProvider: ThemeProviderState) {
  if (themeProvider.theme == 'dark') {
    themeProvider.setTheme('light');
  } else {
    themeProvider.setTheme('dark');
  }
}

export function Menu() {
  const themeProvider = useTheme();
  const dispatch = useContext(AppContext);

  useHotkeys('ctrl+o', () => dispatch({ type: "open_file", dispatch }));
  useHotkeys('ctrl+k', () => toggleDarkMode(themeProvider));

  return (
    <div>
      <Menubar>
        <MenubarMenu>
          <MenubarTrigger>File</MenubarTrigger>
          <MenubarContent>
            <MenubarItem onClick={() => dispatch({ type: "open_file", dispatch })}>
              Open File...
              <MenubarShortcut>Ctrl+O</MenubarShortcut>
            </MenubarItem>
          </MenubarContent>
        </MenubarMenu>
        <MenubarMenu>
          <MenubarTrigger>View</MenubarTrigger>
          <MenubarContent>
            <MenubarCheckboxItem checked={themeProvider.theme == 'dark'} onClick={() => toggleDarkMode(themeProvider)}>
              Dark Mode
              <MenubarShortcut>Ctrl+K</MenubarShortcut>
            </MenubarCheckboxItem>
          </MenubarContent>
        </MenubarMenu>
      </Menubar>
    </div>
  )
}
