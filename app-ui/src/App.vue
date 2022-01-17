<template>
  <section class="section">
    <div id="app" class="container">
      <nav-bar />
      <section class="section">
        <div class="container">
          <router-view />
        </div>
      </section>
    </div>
  </section>
</template>

<script>
import 'bulma/css/bulma.css';
import {
  createClient,
  defaultExchanges,
  provideClient,
  subscriptionExchange,
} from '@urql/vue';
import { createClient as createWsClient } from 'graphql-ws';

import NavBar from './components/NavBar.vue';

export default {
  name: 'App',
  components: {
    NavBar,
  },
  setup() {
    const wsClient = createWsClient({
      url: `wss://${process.env.VUE_APP_API_ADDRESS}/subscriptions`,
    });

    const client = createClient({
      url: `https://${process.env.VUE_APP_API_ADDRESS}/graphql`,
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
