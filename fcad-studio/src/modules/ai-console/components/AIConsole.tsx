import { useState, useEffect } from "preact/hooks";
import { ScrollArea } from "@/modules/core/components/ui/scroll-area";
import { MessageSquare } from "lucide-react";
import { Button } from "@/modules/core/components/ui/button";
import { useTranslation } from "react-i18next";

export function AIConsole() {
  const { t } = useTranslation();

  const [messages, setMessages] = useState<{ role: string; content: string }[]>(
    [],
  );

  // Set initial welcome message depending on language
  useEffect(() => {
    if (messages.length === 0) {
      setMessages([
        {
          role: "assistant",
          content: t("ui.ai_console.welcome"),
        },
      ]);
    }
  }, [t, messages.length]);

  const [input, setInput] = useState("");

  const sendMessage = () => {
    if (!input.trim()) return;
    setMessages([...messages, { role: "user", content: input }]);
    setTimeout(() => {
      setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          content: t("ui.ai_console.mock_response", { input }),
        },
      ]);
    }, 500);
    setInput("");
  };

  return (
    <div className="bg-card flex h-full flex-col">
      <div className="text-muted-foreground flex items-center gap-2 border-b p-2 text-[10px] font-bold tracking-tighter uppercase">
        <MessageSquare className="h-3 w-3" /> {t("ui.ai_console.title")}
      </div>
      <ScrollArea className="flex-1 p-4">
        <div className="space-y-4">
          {messages.map((m, i) => (
            <div
              key={i}
              className={`flex ${m.role === "user" ? "justify-end" : "justify-start"}`}
            >
              <div
                className={`max-w-[80%] rounded-lg p-3 text-xs ${
                  m.role === "user"
                    ? "bg-primary text-primary-foreground"
                    : "bg-muted border"
                }`}
              >
                {m.content}
              </div>
            </div>
          ))}
        </div>
      </ScrollArea>
      <div className="bg-background flex gap-2 border-t p-2">
        <input
          className="bg-muted focus:ring-primary flex-1 rounded border px-3 py-1 text-xs focus:ring-1 focus:outline-none"
          placeholder={t("ui.ai_console.placeholder")}
          value={input}
          onKeyPress={(e) => e.key === "Enter" && sendMessage()}
          onInput={(e) => setInput(e.currentTarget.value)}
        />
        <Button size="sm" onClick={sendMessage} className="h-8">
          {t("ui.ai_console.send")}
        </Button>
      </div>
    </div>
  );
}
