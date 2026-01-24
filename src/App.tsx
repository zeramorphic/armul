import { ReactNode, useEffect, useState } from "react";
import "./App.css";
import { Menu } from "./components/my/Menu";
import { Toaster } from "./components/ui/sonner";
import { useTheme } from "./components/theme-provider";
import { ProcessorContext } from "./lib/ProcessorContext";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia } from "./components/ui/empty";
import { Binary } from "lucide-react";
import { Button } from "./components/ui/button";
import { DispatchContext } from "./lib/DispatchContext";
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
      // Using .reduce here is a little dodgy because we might be pushing to the actionQueue at the same time we're traversing it.
      // To combat this, we loop, popping from the queue and then executing. This fixes any concurrent modification problems.
      var intermediateState = state;
      while (true) {
        const removed = actionQueue.splice(0, 1);
        if (removed.length === 0) {
          break;
        }
        for (const action of removed) {
          intermediateState = performAction(intermediateState, action,
            (err) => { alertSetOpen(true); alertSetContents(err); })
        }
      }
      setState(intermediateState);
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
      <DispatchContext value={dispatch}>
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
      </DispatchContext>
    </>
  );
}
