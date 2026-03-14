import { useAppStore } from "../stores/appStore";
import { triggerCrawl } from "../lib/tauri";

export function SplashScreen() {
  const setIsFirstRun = useAppStore((s) => s.setIsFirstRun);

  const handleGetStarted = async () => {
    localStorage.setItem("cazzmachine_first_run", "false");
    setIsFirstRun(false);
    await triggerCrawl().catch(console.error);
  };

  const previouslyDismissed = localStorage.getItem("cazzmachine_first_run") === "false";
  if (previouslyDismissed) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 bg-cazz-bg flex flex-col items-center p-4 overflow-y-auto">
      <div className="max-w-lg w-full space-y-6 animate-fade-in py-4">
        <div className="text-center space-y-2">
          <h1 className="text-4xl font-black tracking-tighter text-cazz-text uppercase italic">
            cazzmachine
          </h1>
          <p className="text-xs font-mono uppercase tracking-[0.2em] text-cazz-muted">
            la macchina che cazzeggia per te
          </p>
        </div>

        <div className="bg-cazz-surface border border-cazz-border rounded-lg p-4">
          <p className="font-mono text-sm text-cazz-text leading-relaxed">
            It's Monday again. You're back in the office, slaving at the terminal. An icon beckons from the side pane, tantalising, alluring. The browser.
            <br /><br />
            "Screw work!" you grunt, and click.
            <br />
            And click. And click again.
            <br /><br />
            Suddenly your work day is over, and all you have done is browsing with glazed eyes through unfunny jokes, half-heartedly smiling at animal videos of dubiously authentic nature...
            <br /><br />
            If only the curse of curiosity could be lifted. If only someone - something - could bear the burden of cat videos and dank memes - the internet <em className="italic">cazzeggio</em> - in your place.
            <br /><br />
            But now, my friend, there is.
            <br /><br />
            Let the browser icon lie neglected - let the <strong className="font-bold">Cazzmachine</strong> toil! No longer will you be unproductive. No longer will you waste hours consuming machine-generated drivel.
            <br /><br />
            For now comes the era of the <strong className="font-bold">Cazzmachine</strong>.
          </p>
        </div>

        <button
          onClick={handleGetStarted}
          className="w-full py-4 px-6 bg-cazz-accent hover:bg-cazz-accent/80 text-white font-mono text-sm uppercase tracking-wider rounded-lg transition-colors"
        >
          [ BEGIN_DOOMSCROLLING ]
        </button>
      </div>
    </div>
  );
}
