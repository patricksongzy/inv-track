<template>
  <div v-if="live?.length > 0 || base?.length > 0">
    <div class="table-container">
      <table class="table is-fullwidth is-hoverable">
        <thead>
          <tr>
            <th>ID</th>
            <th>Item ID</th>
            <th>Item SKU</th>
            <th>Item</th>
            <th>Location ID</th>
            <th>Location Address</th>
            <th>Location</th>
            <th>Transaction Date</th>
            <th>Quantity</th>
            <th>Comment</th>
            <th>Link</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="transaction in live ?? base" :key="transaction.id">
            <td>{{ transaction.id }}</td>
            <td>{{ transaction.item.id }}</td>
            <td>{{ transaction.item.sku ?? '--' }}</td>
            <td>
              <router-link
                :to="`/items/${transaction.item.id}`"
                class="is-link"
                >{{ transaction.item.name }}</router-link
              >
            </td>
            <td>{{ transaction.location?.id ?? '--' }}</td>
            <td>{{ transaction.location?.address ?? '--' }}</td>
            <td>
              <router-link
                v-if="transaction.location"
                :to="`/locations/${transaction.location.id}`"
                class="is-link"
                >{{ transaction.location.name }}</router-link
              >
              <div v-else>--</div>
            </td>
            <td>{{ transaction.transactionDate?.split('T')[0] ?? '--' }}</td>
            <td>{{ transaction.quantity }}</td>
            <td>{{ transaction.comment ?? '--' }}</td>
            <td>
              <router-link
                :to="`/transactions/${transaction.id}`"
                class="button is-link"
                >View Transaction</router-link
              >
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
  <div v-else>
    <p>No transactions added to the inventory tracking system.</p>
  </div>
</template>
<script>
import { useClientHandle } from '@urql/vue';

export default {
  async setup() {
    const handle = useClientHandle();

    // get the initial transactions
    let result = await handle
      .useQuery({
        query: `
      {
        transactions {
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
    result = JSON.parse(JSON.stringify(result.data.value.transactions));

    // update the transactions live
    const handleSubscription = (transactions = result, response) => {
      const event = response.transactionSubscription;
      if (event.modification === 'CREATE') {
        if (!event.data.transactionDate) {
          return [event.data, ...transactions];
        }
        // naive insert to sorted transaction
        let i = 0;
        for (; i < transactions.length; i++) {
          if (
            transactions[i].transactionDate &&
            new Date(transactions[i].transactionDate) <=
              new Date(event.data.transactionDate)
          ) {
            break;
          }
        }
        transactions.splice(i, 0, event.data);
        return transactions;
      } else if (event.modification === 'UPDATE') {
        // super naive update
        transactions = transactions.filter(
          (transaction) => transaction.id !== event.data.id
        );
        // naive insert to sorted transaction
        let i = 0;
        for (; i < transactions.length; i++) {
          if (
            transactions[i].transactionDate &&
            new Date(transactions[i].transactionDate) <=
              new Date(event.data.transactionDate)
          ) {
            break;
          }
        }
        transactions.splice(i, 0, event.data);
        return transactions;
      } else if (event.modification === 'DELETE') {
        return transactions.filter(
          (transaction) => transaction.id !== event.data.id
        );
      }
    };

    // subscribe to transaction changes
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

    // hack to get things working
    return {
      base: result,
      live: subscription.data,
    };
  },
};
</script>