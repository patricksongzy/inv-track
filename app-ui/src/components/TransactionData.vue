<template>
  <div v-if="transaction">
    <div v-if="!isEdit">
      <h1 class="title">
        Transaction of {{ transaction.item.name }}
        {{ transaction.location ? `at ${transaction.location.name}` : '' }} for
        {{ transaction.quantity }} Units
        <button class="button mr-1" v-on:click="isEdit = true">Edit</button>
        <button class="button is-danger" v-on:click="deleteTransaction">
          Delete
        </button>
      </h1>
      <p><strong>ID</strong>: {{ transaction.id }}</p>
      <p><strong>Item ID</strong>: {{ transaction.item.id }}</p>
      <p><strong>Item SKU</strong>: {{ transaction.item.sku }}</p>
      <p>
        <strong>Item</strong>:
        <router-link :to="`/items/${transaction.item.id}`">{{
          transaction.item.name
        }}</router-link>
      </p>
      <p>
        <strong>Location ID</strong>: {{ transaction.location?.id ?? '--' }}
      </p>
      <p>
        <strong>Location Address</strong>:
        {{ transaction.location?.address ?? '--' }}
      </p>
      <p>
        <strong>Location</strong>:
        <router-link
          v-if="transaction.location"
          :to="`/locations/${transaction.location.id}`"
        >
          {{ transaction.location.name }}
        </router-link>
        <span v-if="!transaction.location">--</span>
      </p>
      <p>
        <strong>Transaction Date</strong>:
        {{ transaction.transactionDate ?? '--' }}
      </p>
      <p><strong>Quantity</strong>: {{ transaction.quantity }}</p>
      <p><strong>Comment</strong>: {{ transaction.comment ?? '--' }}</p>
    </div>
    <div v-else>
      <transaction-change
        v-on:submit="isEdit = false"
        :initialItemId="transaction.item.id"
        :initialLocationId="transaction.location?.id"
        :initialTransactionDate="new Date(transaction.transactionDate)?.toISOString()?.slice(0, 16)"
        :initialQuantity="transaction.quantity"
        :initialComment="transaction.comment"
      />
    </div>
  </div>
  <div v-else>
    <p>Location not found.</p>
  </div>
</template>
<script>
import { useRoute } from 'vue-router';
import { useClientHandle } from '@urql/vue';
import TransactionChange from '../components/TransactionChange.vue';

export default {
  data: () => {
    return {
      isEdit: false,
    };
  },
  components: {
    TransactionChange,
  },
  async setup() {
    const handle = useClientHandle();
    const route = useRoute();
    let id = route.params.id;

    // get the initial transaction
    let result = await handle
      .useQuery({
        query: `
      {
        transaction(id: ${id}) {
          id,
          item {
            id,
            sku,
            name,
          },
          location {
            id,
            name,
            address,
          },
          transactionDate,
          quantity,
          comment,
        }
      }
      `,
      })
      .executeQuery();
    if (result.data?.value?.transaction) {
      result = JSON.parse(JSON.stringify(result.data?.value?.transaction));
      // update the transaction live
      const handleSubscription = (_, response) => {
        const event = response.transactionSubscription;
        if (event.data.id == id) {
          if (event.modification === 'UPDATE') {
            return event.data;
          }
        }
      };

      // subscribe to transaction changes
      // note this is very inefficient
      const subscription = handle.useSubscription(
        {
          query: `
        subscription {
          transactionSubscription {
            modification,
            data {
              id,
              item {
                id,
                sku,
                name,
              },
              location {
                id,
                name,
                address,
              },
              transactionDate,
              quantity,
              comment,
            }
          }
        }
        `,
        },
        handleSubscription
      );

      const deleteTransaction = handle.useMutation(`
        mutation ($id: TransactionId!) {
          deleteTransaction(id: $id) {
            id
          }
        }
      `);

      // hack to get things working
      return {
        base: result,
        live: subscription.data,
        deleteTransaction() {
          deleteTransaction
            .executeMutation({ id: parseInt(id) })
            .then((result) => {
              if (result.error) {
                alert(JSON.stringify(result.error.graphQLErrors[0]));
              } else {
                alert('Deleted!');
                this.$router.push('/transactions');
              }
            });
        },
      };
    } else {
      return {
        base: undefined,
        live: undefined,
      };
    }
  },
  computed: {
    transaction: function () {
      return this.live ?? this.base;
    },
  },
};
</script>