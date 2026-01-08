import { ReactNode, useEffect, useState } from "react";
import "./App.css";
import { Menu } from "./components/my/Menu";
import { Toaster } from "./components/ui/sonner";
import { useTheme } from "./components/theme-provider";
import { ProcessorContext } from "./lib/ProcessorContext";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia } from "./components/ui/empty";
import { Binary } from "lucide-react";
import { Button } from "./components/ui/button";
import { AppContext } from "./lib/AppContext";
import { AlertDialog, AlertDialogContent } from "./components/ui/alert-dialog";
import TabLayout from "./components/my/TabLayout";
import { AppAction, AppDispatch, newAppState, performAction } from "./AppAction";

const actionQueue: AppAction[] = [];

export default function App() {
  const [state, setState] = useState(newAppState());
  const theme = useTheme();

  const [alertOpen, alertSetOpen] = useState(false);
  const [alertContents, alertSetContents] = useState<ReactNode>();

  const [actionsPending, setActionsPending] = useState(false);
  const dispatch: AppDispatch = (action) => {
    actionQueue.push(action);
    setActionsPending(true);
  };

  useEffect(() => {
    if (actionsPending) {
      const finalState = actionQueue.reduce((state, action) =>
        performAction(state, action,
          (err) => { alertSetOpen(true); alertSetContents(err); }),
        state);
      setState(finalState);
      setActionsPending(false);
      actionQueue.length = 0;
    }
  }, [actionsPending]);

  var body;
  if (state.ready) {
    body = <TabLayout />;
  } else {
    body = <Empty>
      <EmptyMedia className="bg-muted p-3">
        <Binary size={28} />
      </EmptyMedia>
      <EmptyHeader>Emulator Ready</EmptyHeader>
      <EmptyDescription>
        Open an ARM assembly file (<span className="font-mono">.s</span>) to load it into the emulator.
      </EmptyDescription>
      <EmptyContent>
        <Button onClick={() => dispatch({ type: "open_file", dispatch })}>Open File</Button>
      </EmptyContent>
    </Empty>;
  }

  return (
    <>
      <AlertDialog open={alertOpen} onOpenChange={alertSetOpen}>
        <AlertDialogContent>
          {alertContents}
        </AlertDialogContent>
      </AlertDialog>
      <AppContext value={dispatch}>
        <ProcessorContext value={state.processor}>
          <main className="container">
            <div className="row">
              <Menu />
            </div>

            <div className={'mainbody row ' + (theme.theme === 'light' ? "flexlayout__theme_light" : "flexlayout__theme_dark")}>
              {body}
            </div>
          </main>
          <Toaster />
        </ProcessorContext>
      </AppContext>
    </>
  );
}
