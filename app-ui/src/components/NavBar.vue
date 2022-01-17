<template>
  <nav class="navbar" role="navigation" aria-label="main navigation">
    <div class="navbar-brand">
      <router-link to="/" class="navbar-item"
        ><strong>Inv-Track</strong></router-link
      >
      <router-link to="/items" class="navbar-item">Items</router-link>
      <router-link to="/locations" class="navbar-item">Locations</router-link>
      <router-link to="/transactions" class="navbar-item"
        >Transactions</router-link
      >
    </div>
    <div class="navbar-end">
      <a v-bind:href="`${protocol}${api_url}/playground`" class="navbar-item"
        >GraphQL Playground</a
      >
    </div>
  </nav>
</template>
<script>
export default {
  data() {
    return {
      api_url: process.env.VUE_APP_API_ADDRESS,
      protocol: process.env.VUE_APP_PROTOCOL,
    };
  },
};
</script>
