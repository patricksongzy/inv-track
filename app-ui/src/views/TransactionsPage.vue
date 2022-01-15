<template>
  <h1>Transactions</h1>
  <Suspense>
    <template #default>
      <transactions-list />
    </template>
    <template #fallback>
      <p>Unable to load transactions.</p>
    </template>
  </Suspense>
</template>
<script>
import TransactionsList from '../components/TransactionsList.vue';

export default {
  components: {
    TransactionsList,
  },
};
</script>