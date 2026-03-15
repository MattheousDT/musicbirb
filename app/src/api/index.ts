import { useMusicbirb } from "@/context/MusicbirbContext";
import {
  useQuery,
  UseQueryOptions,
  UseQueryResult,
} from "@tanstack/react-query";
import { MusicbirbInterface } from "musicbirb-ffi";

type QueryMethod<T extends (...args: any[]) => any> = {
  useQuery: (
    args: Parameters<T>,
    options?: Omit<
      UseQueryOptions<
        Awaited<ReturnType<T>>,
        Error,
        Awaited<ReturnType<T>>,
        any[]
      >,
      "queryKey" | "queryFn"
    >,
  ) => UseQueryResult<Awaited<ReturnType<T>>, Error>;
};

type ApiProxy = {
  [K in keyof MusicbirbInterface]: MusicbirbInterface[K] extends (
    ...args: any[]
  ) => any
    ? QueryMethod<MusicbirbInterface[K]>
    : never;
};

export function useApi() {
  const { core } = useMusicbirb();

  return new Proxy(
    {},
    {
      get(target, prop: string) {
        return {
          useQuery: (args: any[], options?: any) => {
            return useQuery({
              queryKey: [prop, ...(args || [])],
              queryFn: async () => {
                if (!core) return null;
                const method = (core as any)[prop];
                if (typeof method === "function") {
                  return await method.apply(core, args || []);
                }
                throw new Error(`Method ${prop} not found on core`);
              },
              enabled: !!core && (options?.enabled ?? true),
              ...options,
            });
          },
        };
      },
    },
  ) as ApiProxy;
}
