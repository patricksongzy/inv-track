<template>
  <item-list></item-list>
</template>

<script>
import { createClient, defaultExchanges, provideClient, subscriptionExchange } from '@urql/vue';
import { createClient as createWsClient } from 'graphql-ws';

import ItemList from './components/ItemList.vue';

export default {
  name: 'App',
  components: {
    ItemList,
  },
  setup() {
    const wsClient = createWsClient({
      url: 'ws://localhost:8000/subscriptions',
    });

    const client = createClient({
      url: 'http://localhost:8000/graphql',
      exchanges: [
        ...defaultExchanges,
        subscriptionExchange({
          forwardSubscription: (operation) => ({
            subscribe: (sink) => ({
              unsubscribe: wsClient.subscribe(operation, sink),
            }),
          }),
        }),
      ],
    });

    provideClient(client);
  },
};
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
