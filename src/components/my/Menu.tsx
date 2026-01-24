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
import { DispatchContext } from "@/lib/DispatchContext";
import { AppDispatch } from "@/AppAction";
import { AlertDialogCancel, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from "../ui/alert-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";

function toggleDarkMode(themeProvider: ThemeProviderState) {
  if (themeProvider.theme == 'dark') {
    themeProvider.setTheme('light');
  } else {
    themeProvider.setTheme('dark');
  }
}

export function Menu() {
  const themeProvider = useTheme();
  const dispatch = useContext(DispatchContext);

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
        <MenubarMenu>
          <MenubarTrigger>Help</MenubarTrigger>
          <MenubarContent>
            <MenubarItem onClick={() => openAbout(dispatch)}>
              About ARMUL
            </MenubarItem>
          </MenubarContent>
        </MenubarMenu>
      </Menubar>
    </div>
  )
}

function openAbout(dispatch: AppDispatch) {
  const contents = <>
    <AlertDialogHeader>
      <AlertDialogTitle>
        About ARMUL
      </AlertDialogTitle>
      <AlertDialogDescription className="text-sm">
        ARMUL is an ARM7TDMI emulator and graphical debugger.
        The program was inspired by the <a className="cursor-pointer" onClick={() => { openUrl("https://studentnet.cs.manchester.ac.uk/resources/software/komodo/") }}>Komodo</a> software developed by Charlie Brej and Jim Garside.
        <div className="my-4" />
        ARMUL was written by <a className="cursor-pointer" onClick={() => { openUrl("https://zeramorphic.uk") }}>Sky Wilshaw</a>.
        The code is <a className="cursor-pointer" onClick={() => { openUrl("https://github.com/zeramorphic/armul") }}>open-source</a>.
        Please send any issues or bug reports to the repository's <a className="cursor-pointer" onClick={() => { openUrl("https://github.com/zeramorphic/armul/issues") }}>issue tracker</a>.
      </AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel>Close</AlertDialogCancel>
    </AlertDialogFooter>
  </>;
  dispatch({ type: "alert", contents })
}
