import { useRouter } from "next/router";
import React, { useCallback, useContext, useState } from "react";
import update from "immutability-helper";

export interface Toast {
  variant: string,
  message: string,
  title: string
};

export const ToastContext = React.createContext<{ toasts: Toast[], add: (variant: string, e: string, title?: string) => void, addError: (e: string, title?: string) => void, addWarning: (e: string, title?: string) => void, addInfo: (e: string, title?: string) => void, removeToast: (idx: number) => void }>({
  toasts: [],
  add: () => {},
  addError: () => {},
  addWarning: () => {},
  addInfo: () => {},
  removeToast: () => {}
});

export default function ErrorProvider({ children }: { children: React.ReactElement }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const remove = (i: number) => setToasts(update(toasts, { $splice: [[i, 1]] }));
  const add = (variant: string, e: string, title?: string) => setToasts(update(toasts, { $push: [{ title: title || "",  message: e, variant }]  }))

  const contextValue = {
    toasts,
    add: useCallback((variant: string, e: string, title?: string) => add(variant, e, title), []),
    addError: useCallback((e: string, title?: string) => add("danger", e, title), []),
    addWarning: useCallback((e: string, title?: string) => add("warning", e, title), []),
    addInfo: useCallback((e: string, title?: string) => add("primary", e, title), []),
    removeToast: useCallback((i: number) => remove(i), [])
  };

  return (
    <ToastContext.Provider value={contextValue as any}>
      {children}
    </ToastContext.Provider>
  );
}

export function useToasts() {
  const { toasts, add, addError, addWarning, addInfo, removeToast } = useContext(ToastContext);
  return { toasts, add, addError, addWarning, addInfo, removeToast };
}