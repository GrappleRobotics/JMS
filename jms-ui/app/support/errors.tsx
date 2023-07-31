import { useRouter } from "next/router";
import React, { useCallback, useContext, useState } from "react";

export const ErrorContext = React.createContext<{ error: string | null, addError: (e: string) => void, removeError: () => void }>({
  error: null,
  addError: () => {},
  removeError: () => {}
});

export default function ErrorProvider({ children }: { children: React.ReactElement }) {
  const [error, setError] = useState<string | null>(null);

  const removeError = () => setError(null);

  const addError = (error: string) => setError(error);

  const contextValue = {
    error,
    addError: useCallback((error: string) => addError(error), []),
    removeError: useCallback(() => removeError(), [])
  };

  return (
    <ErrorContext.Provider value={contextValue as any}>
      {children}
    </ErrorContext.Provider>
  );
}

export function useErrors() {
  const { error, addError, removeError } = useContext(ErrorContext);
  return { error, addError, removeError };
}